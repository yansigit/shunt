//! Bounded incremental SSE decoding for Gemini response streams.

#[cfg(test)]
mod tests {
    use serde_json::json;

    #[derive(Debug, PartialEq)]
    enum Item {
        Json(serde_json::Value),
        Done,
    }

    struct Decoder;

    impl Decoder {
        fn with_limits(_max_event_bytes: usize, _max_events_per_feed: usize) -> Self {
            Self
        }

        fn push(&mut self, _chunk: &[u8]) -> Result<Vec<Item>, String> {
            Ok(Vec::new())
        }

        fn finish(&mut self) -> Result<(), String> {
            Ok(())
        }
    }

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

        assert!(decoder.push(&wire.as_bytes()[..wave + 1]).unwrap().is_empty());
        let items = decoder.push(&wire.as_bytes()[wave + 1..]).unwrap();

        assert_eq!(
            items,
            vec![Item::Json(json!({"text": "Olá 🌊", "n": 1})), Item::Json(json!({"n": 2}))]
        );
    }

    #[test]
    fn gemini_sse_bounds_accepts_exact_event_and_rejects_plus_one() {
        let frame = b"data: {}";
        let mut exact = Decoder::with_limits(frame.len(), 2);
        let mut exact_wire = frame.to_vec();
        exact_wire.extend_from_slice(b"\n\n");
        assert_eq!(exact.push(&exact_wire).unwrap(), vec![Item::Json(json!({}))]);

        let mut oversized = Decoder::with_limits(frame.len() - 1, 2);
        assert!(oversized.push(&exact_wire).unwrap_err().contains("exceeded"));
        assert!(oversized.push(b"data: {}\n\n").unwrap_err().contains("disposed"));
    }

    #[test]
    fn gemini_sse_bounds_rejects_event_amplification_and_disposes() {
        let mut decoder = Decoder::with_limits(64, 2);
        assert!(decoder
            .push(b": one\n\n: two\r\n\r\n: three\r\n\n")
            .unwrap_err()
            .contains("event limit"));
        assert!(decoder.push(b"data: {}\n\n").unwrap_err().contains("disposed"));
    }

    #[test]
    fn gemini_sse_bounds_rejects_invalid_utf8_and_json_then_disposes() {
        for bad in [b"data: \xff\n\n".as_slice(), b"data: nope\n\n".as_slice()] {
            let mut decoder = Decoder::with_limits(64, 2);
            assert!(decoder.push(bad).is_err());
            assert!(decoder.push(b"data: {}\n\n").unwrap_err().contains("disposed"));
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
