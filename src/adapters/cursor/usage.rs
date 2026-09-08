//! Active Run usage, not StreamUnifiedChatWithTools. Schema source: OpenCodex
//! 055c3ecf0de6c35f59195fc434d6b08525182b7f agent_pb.ts, 2026-09-07.
//! TokenDeltaUpdate.tokens is additive int32 output. Checkpoint token_details
//! used_tokens is absolute uint32 context occupancy. Input is derived; cache
//! counts and absent usage are unavailable, not measured zero.
use super::{response::CursorStreamEvent, wire};

#[derive(Default)]
pub(super) struct Tracker {
    context: Option<u64>,
    output: u64,
}

fn bytes(buf: &[u8], tag: u32) -> Result<Option<&[u8]>, String> {
    let mut value = None;
    for field in wire::fields(buf) {
        let field = field.map_err(|e| format!("cursor usage protobuf: {e}"))?;
        if field.number == tag {
            if field.wire != 2 || value.is_some() {
                return Err("cursor usage: duplicate or wrong-wire message".into());
            }
            value = Some(field.bytes);
        }
    }
    Ok(value)
}

fn uint(buf: &[u8], tag: u32) -> Result<u64, String> {
    let mut value = None;
    for field in wire::fields(buf) {
        let field = field.map_err(|e| format!("cursor usage protobuf: {e}"))?;
        if field.number == tag {
            if field.wire != 0 || value.is_some() {
                return Err("cursor usage: duplicate or wrong-wire counter".into());
            }
            value = Some(field.varint);
        }
    }
    Ok(value.unwrap_or(0))
}

impl Tracker {
    pub fn accept(&mut self, payload: &[u8]) -> Result<Option<CursorStreamEvent>, String> {
        let mut changed = false;
        if let Some(interaction) = bytes(payload, 1)? {
            if let Some(delta) = bytes(interaction, 8)? {
                let delta = uint(delta, 1)?;
                if delta > i32::MAX as u64 {
                    return Err("cursor usage: negative or overflowing output token delta".into());
                }
                self.output = self
                    .output
                    .checked_add(delta)
                    .ok_or("cursor usage output counter overflow")?;
                changed = true;
            }
        }
        if let Some(checkpoint) = bytes(payload, 3)? {
            if let Some(details) = bytes(checkpoint, 5)? {
                let used = uint(details, 1)?;
                if used > u32::MAX as u64 {
                    return Err("cursor usage: context counter overflow".into());
                }
                self.context = Some(used);
                changed = true;
            }
        }
        Ok(changed.then_some(CursorStreamEvent::Usage {
            input_tokens: self.context.unwrap_or(0).saturating_sub(self.output),
            output_tokens: self.output,
            cache_read_tokens: 0,
            cache_write_tokens: 0,
        }))
    }
}

pub(super) fn ended(payload: &[u8]) -> Result<bool, String> {
    let Some(interaction) = bytes(payload, 1)? else {
        return Ok(false);
    };
    let Some(ended) = bytes(interaction, 14)? else {
        return Ok(false);
    };
    for field in wire::fields(ended) {
        field.map_err(|e| format!("cursor terminal protobuf: {e}"))?;
    }
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapters::cursor::{
        agent::{field_ld, field_varint},
        test_frames::active_wire,
    };

    #[test]
    fn cursor_usage_relay_absolute_not_additive_and_absence_is_none() {
        let mut tracker = Tracker::default();
        assert!(tracker.accept(b"").unwrap().is_none());
        for payload in [
            active_wire::checkpoint(10000),
            active_wire::delta(42),
            active_wire::checkpoint(10300),
        ] {
            tracker.accept(&payload).unwrap();
        }
        assert_eq!(tracker.context, Some(10300));
        assert_eq!(tracker.output, 42);
        assert!(ended(&active_wire::ended()).unwrap());
    }
    #[test]
    fn cursor_usage_relay_rejects_negative_overflow_and_corruption() {
        for payload in [
            active_wire::delta(-1),
            field_ld(1, &field_ld(8, &field_varint(1, i32::MAX as u64 + 1))),
            field_ld(3, &field_ld(5, &field_varint(1, u32::MAX as u64 + 1))),
            field_ld(1, &field_ld(8, &[0x80])),
        ] {
            assert!(Tracker::default().accept(&payload).is_err());
        }
    }
}
