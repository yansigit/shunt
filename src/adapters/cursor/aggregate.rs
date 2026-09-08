//! Bounded, ordered JSON content for the active Run response.
use serde_json::{json, Value};

pub(super) struct Content {
    blocks: Vec<Value>,
    remaining: usize,
}
impl Content {
    pub fn new() -> Self {
        Self {
            blocks: Vec::new(),
            remaining: super::connect::MAX_DECOMPRESSED_FRAME_BYTES as usize,
        }
    }

    fn charge(&mut self, bytes: usize) -> Result<(), String> {
        let cost = bytes
            .checked_mul(6)
            .and_then(|n| n.checked_add(64))
            .ok_or("cursor aggregate size overflow")?;
        self.remaining = self
            .remaining
            .checked_sub(cost)
            .ok_or("cursor aggregate response budget exceeded")?;
        Ok(())
    }

    pub fn delta(&mut self, thinking: bool, text: &str) -> Result<(), String> {
        if text.is_empty() {
            return Ok(());
        }
        self.charge(text.len())?;
        let (kind, key) = if thinking {
            ("thinking", "thinking")
        } else {
            ("text", "text")
        };
        if let Some(last) = self.blocks.last_mut().filter(|block| block["type"] == kind) {
            if let Some(Value::String(previous)) = last.get_mut(key) {
                previous.push_str(text);
                return Ok(());
            }
        }
        let mut block = json!({"type":kind, key:text});
        if thinking {
            block["signature"] = json!("");
        }
        self.blocks.push(block);
        Ok(())
    }

    pub fn tool(&mut self, id: &str, name: &str, input: &str) -> Result<(), String> {
        let size = id
            .len()
            .checked_add(name.len())
            .and_then(|n| n.checked_add(input.len()))
            .ok_or("cursor aggregate size overflow")?;
        self.charge(size)?;
        let input: Value =
            serde_json::from_str(input).map_err(|_| "cursor tool arguments are invalid JSON")?;
        if !input.is_object() {
            return Err("cursor tool arguments must be an object".into());
        }
        self.blocks
            .push(json!({"type":"tool_use","id":id,"name":name,"input":input}));
        Ok(())
    }

    pub fn finish(mut self) -> Vec<Value> {
        if self.blocks.is_empty() {
            self.blocks.push(json!({"type":"text","text":""}));
        }
        self.blocks
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn cursor_output_parity_aggregate_cap_plus_one_and_invalid_arguments() {
        let mut content = Content {
            blocks: Vec::new(),
            remaining: 70,
        };
        content.delta(false, "x").unwrap();
        assert!(content.delta(false, "y").is_err());
        assert_eq!(
            content.finish(),
            json!([{"type":"text","text":"x"}])
                .as_array()
                .unwrap()
                .clone()
        );
        let mut content = Content {
            blocks: Vec::new(),
            remaining: 69,
        };
        assert!(content.delta(true, "x").is_err());
        for invalid in ["{", "null", "[]"] {
            assert!(Content::new().tool("authentic", "Read", invalid).is_err());
        }
    }
}
