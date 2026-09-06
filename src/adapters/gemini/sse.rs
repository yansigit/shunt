//! Bounded incremental SSE decoding for Gemini response streams.

use serde_json::Value;

pub(super) const MAX_EVENT_BYTES: usize = 8 * 1024 * 1024;
pub(super) const MAX_EVENTS_PER_FEED: usize = 256;

#[derive(Debug, PartialEq)]
pub(super) enum Item {
    Json(Value),
    Done,
}

pub(super) struct Decoder {
    buffer: Vec<u8>,
    max_event_bytes: usize,
    max_events_per_feed: usize,
    done: bool,
    disposed: bool,
}

impl Default for Decoder {
    fn default() -> Self {
        Self::with_limits(MAX_EVENT_BYTES, MAX_EVENTS_PER_FEED)
    }
}

impl Decoder {
    pub(super) fn with_limits(max_event_bytes: usize, max_events_per_feed: usize) -> Self {
        assert!(max_event_bytes > 0);
        assert!(max_events_per_feed > 0);
        Self {
            buffer: Vec::new(),
            max_event_bytes,
            max_events_per_feed,
            done: false,
            disposed: false,
        }
    }

    fn fail(&mut self, message: impl Into<String>) -> String {
        self.buffer.clear();
        self.disposed = true;
        message.into()
    }

    pub(super) fn push(&mut self, chunk: &[u8]) -> Result<Vec<Item>, String> {
        if self.disposed {
            return Err("Gemini SSE parser is disposed".to_string());
        }
        let mut items = Vec::new();
        let mut frames_seen = 0usize;
        for &byte in chunk {
            self.buffer.push(byte);
            if let Some(delimiter_len) = terminal_delimiter_len(&self.buffer) {
                let frame_len = self.buffer.len() - delimiter_len;
                if frame_len > self.max_event_bytes {
                    return Err(self.fail(format!(
                        "Gemini SSE event exceeded {} bytes",
                        self.max_event_bytes
                    )));
                }
                if frames_seen >= self.max_events_per_feed {
                    return Err(self.fail(format!(
                        "Gemini SSE chunk exceeded {} event limit",
                        self.max_events_per_feed
                    )));
                }
                frames_seen += 1;
                let frame = self.buffer[..frame_len].to_vec();
                self.buffer.clear();
                match parse_frame(&frame) {
                    Ok(Some(Item::Json(_))) if self.done => {
                        return Err(self.fail("Gemini SSE data arrived after [DONE]"));
                    }
                    Ok(Some(Item::Done)) if self.done => {
                        return Err(self.fail("Gemini SSE contained duplicate [DONE]"));
                    }
                    Ok(Some(Item::Done)) => {
                        self.done = true;
                        items.push(Item::Done);
                    }
                    Ok(Some(item)) => items.push(item),
                    Ok(None) => {}
                    Err(error) => return Err(self.fail(error)),
                }
            } else if retained_candidate_len(&self.buffer) > self.max_event_bytes {
                return Err(self.fail(format!(
                    "Gemini SSE event exceeded {} bytes",
                    self.max_event_bytes
                )));
            }
        }
        Ok(items)
    }

    pub(super) fn finish(&mut self) -> Result<(), String> {
        if self.disposed {
            return Err("Gemini SSE parser is disposed".to_string());
        }
        self.disposed = true;
        if self.buffer.iter().all(u8::is_ascii_whitespace) {
            self.buffer.clear();
            return Ok(());
        }
        Err(self.fail("Gemini SSE ended with an unterminated event frame"))
    }
}

fn retained_candidate_len(buffer: &[u8]) -> usize {
    const PREFIXES: &[&[u8]] = &[b"\r", b"\n", b"\r\n", b"\n\r", b"\r\n\r", b"\n\r\n"];
    let delimiter_prefix = PREFIXES
        .iter()
        .filter(|prefix| buffer.ends_with(prefix))
        .map(|prefix| prefix.len())
        .max()
        .unwrap_or(0);
    buffer.len().saturating_sub(delimiter_prefix)
}

fn terminal_delimiter_len(buffer: &[u8]) -> Option<usize> {
    [b"\r\n\r\n".as_slice(), b"\n\r\n", b"\r\n\n", b"\n\n"]
        .into_iter()
        .find(|delimiter| buffer.ends_with(delimiter))
        .map(<[u8]>::len)
}

fn parse_frame(frame: &[u8]) -> Result<Option<Item>, String> {
    let frame =
        std::str::from_utf8(frame).map_err(|_| "invalid UTF-8 in Gemini SSE event".to_string())?;
    let mut data = Vec::new();
    for line in frame.split('\n') {
        let line = line.strip_suffix('\r').unwrap_or(line);
        if line.starts_with(':') {
            continue;
        }
        let Some((field, value)) = line.split_once(':') else {
            continue;
        };
        if field == "data" {
            data.push(value.strip_prefix(' ').unwrap_or(value));
        }
    }
    let data = data.join("\n");
    if data.is_empty() {
        return Ok(None);
    }
    if data == "[DONE]" {
        return Ok(Some(Item::Done));
    }
    serde_json::from_str(&data)
        .map(Item::Json)
        .map(Some)
        .map_err(|_| "invalid JSON in Gemini SSE event".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn gemini_sse_bounds_preserves_splits_framing_and_event_order() {
        let wire = concat!(
            ": comment\r\n",
            "event: message\r\n",
            "unknown: ignored\r\n",
            "data: {\"text\":\"Olá ",
            "🌊\",\r\n",
            "data: \"n\":1}\r\n\r\n",
            "data:{\"n\":2}\n\r\n",
        );
        let wave = wire.find('🌊').unwrap();
        let mut decoder = Decoder::with_limits(128, 4);

        assert!(decoder
            .push(&wire.as_bytes()[..wave + 1])
            .unwrap()
            .is_empty());
        let items = decoder.push(&wire.as_bytes()[wave + 1..]).unwrap();

        assert_eq!(
            items,
            vec![
                Item::Json(json!({"text": "Olá 🌊", "n": 1})),
                Item::Json(json!({"n": 2}))
            ]
        );
    }

    #[test]
    fn gemini_sse_bounds_accepts_exact_event_and_rejects_plus_one() {
        let frame = b"data: {}";
        let mut exact = Decoder::with_limits(frame.len(), 2);
        let mut exact_wire = frame.to_vec();
        exact_wire.extend_from_slice(b"\n\n");
        assert_eq!(
            exact.push(&exact_wire).unwrap(),
            vec![Item::Json(json!({}))]
        );

        let mut oversized = Decoder::with_limits(frame.len() - 1, 2);
        assert!(oversized
            .push(&exact_wire)
            .unwrap_err()
            .contains("exceeded"));
        assert!(oversized
            .push(b"data: {}\n\n")
            .unwrap_err()
            .contains("disposed"));
    }

    #[test]
    fn gemini_sse_bounds_rejects_event_amplification_and_disposes() {
        let mut decoder = Decoder::with_limits(64, 2);
        assert!(decoder
            .push(b": one\n\n: two\r\n\r\n: three\r\n\n")
            .unwrap_err()
            .contains("event limit"));
        assert!(decoder
            .push(b"data: {}\n\n")
            .unwrap_err()
            .contains("disposed"));
    }

    #[test]
    fn gemini_sse_bounds_rejects_invalid_utf8_and_json_then_disposes() {
        for bad in [b"data: \xff\n\n".as_slice(), b"data: nope\n\n".as_slice()] {
            let mut decoder = Decoder::with_limits(64, 2);
            assert!(decoder.push(bad).is_err());
            assert!(decoder
                .push(b"data: {}\n\n")
                .unwrap_err()
                .contains("disposed"));
        }
    }

    #[test]
    fn gemini_sse_bounds_requires_clean_termination() {
        let mut clean = Decoder::with_limits(64, 2);
        assert!(clean.push(b" \r\n").unwrap().is_empty());
        clean.finish().unwrap();

        let mut cut = Decoder::with_limits(64, 2);
        assert!(cut.push(b"data: {\"n\":1}").unwrap().is_empty());
        assert!(cut.finish().unwrap_err().contains("unterminated"));
        assert!(cut.push(b"\n\n").unwrap_err().contains("disposed"));
    }

    #[test]
    fn gemini_sse_bounds_treats_empty_data_as_noop_and_done_as_terminal_boundary() {
        let mut decoder = Decoder::with_limits(64, 4);
        assert_eq!(
            decoder.push(b"data:\n\ndata: [DONE]\n\n").unwrap(),
            vec![Item::Done]
        );
        assert!(decoder
            .push(b"data: {\"late\":true}\n\n")
            .unwrap_err()
            .contains("after [DONE]"));
    }
}
