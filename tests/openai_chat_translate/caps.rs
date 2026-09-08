// Compile the actual private decoder source; no duplicate parser or public
// production test hook is introduced just to exercise framing boundaries.
#[path = "../../src/adapters/openai_chat/sse.rs"]
mod decoder;
use decoder::{Decoder, Item, MAX_EVENT_BYTES};
use serde_json::json;
use shunt::model::openai_chat_response::OpenAiChatSseMachine;

fn event_probe(size: usize) -> Result<(), String> {
    let prefix = "data: {\"padding\":\"";
    let suffix = "\"}";
    let frame = format!(
        "{prefix}{}{suffix}\n\n",
        "x".repeat(size - prefix.len() - suffix.len())
    );
    let mut decoder = Decoder::default();
    let (_, item) = decoder.push_one(frame.as_bytes())?;
    let Some(Item::Json(value)) = item else {
        panic!("expected padded JSON frame")
    };
    assert_eq!(
        value["padding"].as_str().unwrap().len(),
        size - prefix.len() - suffix.len()
    );
    assert_eq!(decoder.push_one(b"data: [DONE]\n\n")?.1, Some(Item::Done));
    decoder.finish()
}
#[test]
fn cap_event_minus_one() {
    event_probe(MAX_EVENT_BYTES - 1).unwrap();
}
#[test]
fn cap_event_at() {
    event_probe(MAX_EVENT_BYTES).unwrap();
}
#[test]
fn cap_event_plus_one() {
    assert!(event_probe(MAX_EVENT_BYTES + 1)
        .unwrap_err()
        .contains("exceeded"));
}

fn residual_probe(size: usize) -> Result<(), String> {
    let frame = format!(":{}é", "x".repeat(size - 3));
    assert_eq!(frame.len(), size);
    let mut decoder = Decoder::default();
    // The final byte completes a multibyte character; until then all retained
    // payload bytes count toward the cap, without requiring valid UTF-8 yet.
    assert!(decoder.push_one(&frame.as_bytes()[..size - 1])?.1.is_none());
    assert!(decoder.push_one(&frame.as_bytes()[size - 1..])?.1.is_none());
    assert!(decoder.push_one(b"\n\n")?.1.is_none());
    assert_eq!(decoder.push_one(b"data: [DONE]\n\n")?.1, Some(Item::Done));
    decoder.finish()
}
#[test]
fn cap_residual_minus_one() {
    residual_probe(MAX_EVENT_BYTES - 1).unwrap();
}
#[test]
fn cap_residual_at() {
    residual_probe(MAX_EVENT_BYTES).unwrap();
}
#[test]
fn cap_residual_plus_one() {
    assert!(residual_probe(MAX_EVENT_BYTES + 1)
        .unwrap_err()
        .contains("exceeded"));
}

fn aggregate_probe(size: usize) -> Result<(), String> {
    let mut machine = OpenAiChatSseMachine::new_for_upstream("m");
    let first = size / 2;
    for count in [first, size - first] {
        machine
            .process_chunk_checked(&json!({"choices":[{"delta":{"content":"x".repeat(count)}}]}))
            .map_err(|e| e.to_string())?;
    }
    machine
        .process_chunk_checked(&json!({"choices":[{"delta":{},"finish_reason":"stop"}]}))
        .map_err(|e| e.to_string())?;
    machine
        .transport_close_checked()
        .map_err(|e| e.to_string())?;
    assert_eq!(
        machine.final_json_checked().unwrap()["content"][0]["text"]
            .as_str()
            .unwrap()
            .len(),
        size
    );
    Ok(())
}
#[test]
fn cap_aggregate_minus_one() {
    aggregate_probe(8 * 1024 * 1024 - 1).unwrap();
}
#[test]
fn cap_aggregate_at() {
    aggregate_probe(8 * 1024 * 1024).unwrap();
}
#[test]
fn cap_aggregate_plus_one() {
    assert!(aggregate_probe(8 * 1024 * 1024 + 1)
        .unwrap_err()
        .contains("limit"));
}
