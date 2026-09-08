//! Strict source-derived NDJSON framing. No SSE stripping or EOF repair.
use serde_json::Value;

pub(crate) const MAX_RECORD_BYTES: usize = 1024 * 1024;
pub(crate) const MAX_WIRE_BYTES: usize = 32 * 1024 * 1024;

#[derive(Default)]
pub(crate) struct Decoder {
    residual: Vec<u8>,
    total: usize,
    failed: bool,
}
impl Decoder {
    pub(crate) fn feed(&mut self, bytes: &[u8]) -> Result<Vec<Value>, &'static str> {
        let result = self.feed_inner(bytes);
        if result.is_err() {
            self.failed = true;
            self.residual.clear();
        }
        result
    }
    fn feed_inner(&mut self, bytes: &[u8]) -> Result<Vec<Value>, &'static str> {
        if self.failed {
            return Err("NDJSON decoder already failed");
        }
        self.total = self
            .total
            .checked_add(bytes.len())
            .ok_or("NDJSON wire size overflow")?;
        if self.total > MAX_WIRE_BYTES {
            return Err("NDJSON wire budget exceeded");
        }
        let mut records = Vec::new();
        for &byte in bytes {
            if byte == b'\n' {
                let raw = if self.residual.last() == Some(&b'\r') {
                    &self.residual[..self.residual.len() - 1]
                } else {
                    &self.residual
                };
                let text = std::str::from_utf8(raw).map_err(|_| "invalid NDJSON UTF-8")?;
                let value: Value =
                    serde_json::from_str(text).map_err(|_| "invalid NDJSON record")?;
                if !value.is_object() {
                    return Err("NDJSON record must be an object");
                }
                records.push(value);
                self.residual.clear();
            } else {
                if self.residual.len() >= MAX_RECORD_BYTES {
                    return Err("NDJSON record budget exceeded");
                }
                self.residual.push(byte);
            }
        }
        Ok(records)
    }
    pub(crate) fn finish(&self) -> Result<(), &'static str> {
        if self.failed || !self.residual.is_empty() {
            return Err("incomplete NDJSON record at EOF");
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn command_code_bounds_record_crlf_at_cap() {
        let mut bytes = vec![b' '; MAX_RECORD_BYTES - 2];
        bytes.extend_from_slice(b"{}\r\n");
        let mut decoder = Decoder::default();
        let result = decoder.feed(&bytes);
        assert!(result.is_ok(), "at-cap JSON record must accept CRLF framing");
        assert_eq!(result.unwrap(), vec![serde_json::json!({})]);
        assert!(decoder.finish().is_ok());
    }
    #[test]
    fn command_code_tracer_decoder_bounds_and_utf8() {
        for size in [MAX_RECORD_BYTES - 1, MAX_RECORD_BYTES, MAX_RECORD_BYTES + 1] {
            let mut bytes = vec![b' '; size - 2];
            bytes.extend_from_slice(b"{}\n");
            let mut decoder = Decoder::default();
            assert_eq!(decoder.feed(&bytes).is_ok(), size <= MAX_RECORD_BYTES);
        }
        let mut decoder = Decoder::default();
        let wire = "{\"text\":\"é\"}\r\n".as_bytes();
        let mut records = Vec::new();
        for byte in wire {
            records.extend(decoder.feed(&[*byte]).unwrap());
        }
        assert_eq!(records[0]["text"], "é");
        assert!(decoder.finish().is_ok());
        for bad in [
            b"\xff\n".as_slice(),
            b"null\n",
            b"[]\n",
            b"\n",
            b"data: {}\n",
        ] {
            let mut decoder = Decoder::default();
            assert!(decoder.feed(bad).is_err());
            assert!(
                decoder.feed(b"{}\n").is_err(),
                "failed decoder must not recover"
            );
            assert!(decoder.finish().is_err());
        }
        let mut decoder = Decoder::default();
        assert!(decoder.feed(b"{}").is_ok());
        assert!(decoder.finish().is_err());
    }
}
