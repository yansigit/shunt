//! Strict decoding for the active Run tool boundary. Unknown protobuf fields
//! remain compatible; malformed known values never become Null/empty defaults.
use super::wire::{self, Field};
use serde_json::{Map, Value};

pub(super) struct Tool {
    pub id: String,
    pub name: String,
    pub input_json: String,
}

struct Budget(usize);
impl Budget {
    fn take(&mut self, bytes: usize) -> Result<(), String> {
        self.0 = self
            .0
            .checked_sub(bytes)
            .ok_or("cursor protobuf aggregate budget exceeded")?;
        Ok(())
    }
    fn text<'a>(&mut self, bytes: &'a [u8]) -> Result<&'a str, String> {
        // Six bytes per input byte bounds JSON escaping, before allocation.
        self.take(
            bytes
                .len()
                .checked_mul(6)
                .ok_or("cursor protobuf size overflow")?,
        )?;
        std::str::from_utf8(bytes).map_err(|_| "cursor protobuf contains invalid UTF-8".into())
    }
}

fn fields(bytes: &[u8]) -> impl Iterator<Item = Result<Field<'_>, String>> {
    wire::fields(bytes).map(|f| f.map_err(|e| format!("cursor protobuf: {e}")))
}

fn message(bytes: &[u8], tag: u32) -> Result<Option<&[u8]>, String> {
    let mut found = None;
    for field in fields(bytes) {
        let field = field?;
        if field.number == tag {
            if field.wire != 2 || found.is_some() {
                return Err("cursor protobuf duplicate or wrong-wire message".into());
            }
            found = Some(field.bytes);
        }
    }
    Ok(found)
}

fn required(bytes: &[u8], tag: u32) -> Result<&[u8], String> {
    message(bytes, tag)?.ok_or_else(|| "cursor protobuf missing required field".into())
}

fn value(bytes: &[u8], depth: usize, budget: &mut Budget) -> Result<Value, String> {
    if depth >= 64 {
        return Err("cursor protobuf nesting limit exceeded".into());
    }
    budget.take(64)?; // Conservatively bounds node and JSON delimiter overhead.
    let mut result = None;
    for field in fields(bytes) {
        let field = field?;
        if !(1..=6).contains(&field.number) {
            continue;
        }
        if result.is_some() {
            return Err("cursor protobuf Value has multiple kinds".into());
        }
        result = Some(match (field.number, field.wire) {
            (1, 0) if field.varint == 0 => Value::Null,
            (2, 1) => {
                let number = f64::from_le_bytes(
                    field
                        .bytes
                        .try_into()
                        .map_err(|_| "cursor protobuf invalid double")?,
                );
                Value::Number(
                    serde_json::Number::from_f64(number)
                        .ok_or("cursor protobuf non-finite number")?,
                )
            }
            (3, 2) => Value::String(budget.text(field.bytes)?.to_owned()),
            (4, 0) => Value::Bool(field.varint != 0),
            (5, 2) => Value::Object(map(field.bytes, 1, depth + 1, budget)?),
            (6, 2) => {
                let mut values = Vec::new();
                for item in fields(field.bytes) {
                    let item = item?;
                    if item.number == 1 {
                        if item.wire != 2 {
                            return Err("cursor protobuf wrong-wire list item".into());
                        }
                        values.push(value(item.bytes, depth + 1, budget)?);
                    }
                }
                Value::Array(values)
            }
            _ => return Err("cursor protobuf invalid Value kind or wire encoding".into()),
        });
    }
    result.ok_or_else(|| "cursor protobuf Value has no representable kind".into())
}

fn map(
    bytes: &[u8],
    tag: u32,
    depth: usize,
    budget: &mut Budget,
) -> Result<Map<String, Value>, String> {
    let mut values = Map::new();
    for entry in fields(bytes) {
        let entry = entry?;
        if entry.number != tag {
            continue;
        }
        if entry.wire != 2 {
            return Err("cursor protobuf wrong-wire map entry".into());
        }
        // Protobuf map keys are scalar strings: omission legitimately means
        // the empty key. A missing Value message is not a representable kind.
        let key = budget
            .text(message(entry.bytes, 1)?.unwrap_or_default())?
            .to_owned();
        let value = value(required(entry.bytes, 2)?, depth, budget)?;
        if values.insert(key, value).is_some() {
            return Err("cursor protobuf duplicate map key".into());
        }
    }
    Ok(values)
}

fn decode(bytes: &[u8]) -> Result<Option<Tool>, String> {
    let limit = super::connect::MAX_DECOMPRESSED_FRAME_BYTES as usize;
    if bytes.len() > limit {
        return Err("cursor protobuf frame budget exceeded".into());
    }
    let mut budget = Budget(limit);
    let mut known_messages = 0;
    for field in fields(bytes) {
        let field = field?;
        if (1..=4).contains(&field.number) {
            known_messages += 1;
            if field.wire != 2 || known_messages > 1 {
                return Err("cursor protobuf ambiguous or wrong-wire server message".into());
            }
        }
    }
    // Validate known text envelopes even when no tool call is present.
    if let Some(interaction) = message(bytes, 1)? {
        for tag in [1, 4] {
            if let Some(delta) = message(interaction, tag)? {
                if let Some(text) = message(delta, 1)? {
                    budget.text(text)?;
                }
            }
        }
    }
    let Some(exec) = message(bytes, 2)? else {
        return Ok(None);
    };
    let Some(args) = message(exec, 11)? else {
        return Ok(None);
    };
    for tag in [1, 3, 4, 5] {
        if let Some(text) = message(args, tag)? {
            budget.text(text)?;
        }
    }
    let name = message(args, 5)?
        .or(message(args, 1)?)
        .ok_or("cursor MCP tool name is missing")?;
    let name = budget.text(name)?;
    if name.is_empty() {
        return Err("cursor MCP tool name is empty".into());
    }
    let id = message(args, 3)?.ok_or("cursor: native MCP call has no authentic tool_call_id")?;
    let id = budget.text(id)?;
    if id.is_empty() || id.len() > 512 {
        return Err("cursor: native MCP call has no authentic tool_call_id".into());
    }
    if let Some(provider) = message(args, 4)? {
        budget.text(provider)?;
    }
    let input = map(args, 2, 0, &mut budget)?;
    let input_json =
        serde_json::to_string(&input).map_err(|_| "cursor MCP arguments cannot be serialized")?;
    Ok(Some(Tool {
        id: id.into(),
        name: name.into(),
        input_json,
    }))
}

pub(super) async fn tool(bytes: bytes::Bytes) -> Result<Option<Tool>, String> {
    // Reuse the bounded response-work pool for large frames. No new runtime,
    // semaphore, durable state or whole-response buffering is introduced.
    if bytes.len() <= super::connect::INLINE_GZIP_OUTPUT_BYTES {
        return decode(&bytes);
    }
    super::offload::spawn_bounded_gzip(move || decode(&bytes))
        .await
        .map_err(|e| format!("cursor protobuf decode task: {e}"))?
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapters::cursor::agent::{field_ld, field_str};

    #[test]
    fn cursor_proto_wire_strict_depth_and_aggregate_cap_plus_one() {
        let mut encoded = field_str(3, "x");
        for _ in 0..63 {
            encoded = field_ld(6, &field_ld(1, &encoded));
        }
        assert!(value(&encoded, 0, &mut Budget(64 * 1024)).is_ok());
        encoded = field_ld(6, &field_ld(1, &encoded));
        assert!(value(&encoded, 0, &mut Budget(64 * 1024)).is_err());
        let encoded = field_str(3, "x");
        assert!(value(&encoded, 0, &mut Budget(70)).is_ok());
        assert!(value(&encoded, 0, &mut Budget(69)).is_err());
    }

    #[test]
    fn cursor_proto_wire_strict_valid_typed_values_and_unknown_fields() {
        use crate::adapters::cursor::agent::{encode_protobuf_value, field_varint};
        let expected = serde_json::json!({"":null,"count":1.5,"ok":false,"items":["héllo",true,{"key":"value"}]});
        let mut encoded = encode_protobuf_value(&expected);
        encoded.extend(field_varint(99, 7));
        assert_eq!(value(&encoded, 0, &mut Budget(65536)).unwrap(), expected);
        let default_key = field_ld(5, &field_ld(1, &field_ld(2, &field_str(3, "empty key"))));
        assert_eq!(
            value(&default_key, 0, &mut Budget(65536)).unwrap(),
            serde_json::json!({"": "empty key"})
        );
        let mut nan = vec![0x11];
        nan.extend(f64::NAN.to_le_bytes());
        assert!(value(&nan, 0, &mut Budget(65536)).is_err());
        let entry = [field_str(1, "key"), field_ld(2, &field_str(3, "value"))].concat();
        let duplicate = field_ld(5, &[field_ld(1, &entry), field_ld(1, &entry)].concat());
        assert!(value(&duplicate, 0, &mut Budget(65536)).is_err());
    }

    #[test]
    fn cursor_proto_wire_strict_rejects_invalid_text_and_ambiguous_envelopes() {
        let invalid_text = field_ld(1, &field_ld(1, &field_ld(1, &[255])));
        assert!(decode(&invalid_text).is_err());
        let ambiguous = [field_ld(1, &[]), field_ld(2, &[])].concat();
        assert!(decode(&ambiguous).is_err());
    }

    #[tokio::test]
    #[allow(clippy::await_holding_lock)] // Shared pool observer spans test runtimes.
    async fn cursor_proto_wire_strict_large_arguments_match_inline_decoder() {
        let _observer = super::super::offload::OFFLOAD_OBSERVER
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let text = "x".repeat(super::super::connect::INLINE_GZIP_OUTPUT_BYTES + 1);
        let entry = [field_str(1, "content"), field_ld(2, &field_str(3, &text))].concat();
        let args = [
            field_str(1, "Write"),
            field_str(3, "authentic"),
            field_ld(2, &entry),
        ]
        .concat();
        let frame = bytes::Bytes::from(field_ld(2, &field_ld(11, &args)));
        let expected = decode(&frame).unwrap().unwrap();
        let actual = tool(frame).await.unwrap().unwrap();
        assert_eq!(actual.id, expected.id);
        assert_eq!(actual.name, expected.name);
        assert_eq!(actual.input_json, expected.input_json);
    }
}
