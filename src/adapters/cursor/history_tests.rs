use super::*;
use serde_json::json;

fn fixture() -> Value {
    json!({"metadata":{"session_id":"session-A"},"messages":[
        {"role":"user","content":"Read the file"},
        {"role":"assistant","content":[{"type":"tool_use","id":"authentic-17","name":"Read","input":{"path":"a.txt"}}]},
        {"role":"user","content":[{"type":"tool_result","tool_use_id":"authentic-17","content":"file contents"}]}
    ]})
}

fn fields(bytes: &[u8], number: u32) -> Vec<&[u8]> {
    super::super::wire::fields(bytes)
        .map(Result::unwrap)
        .filter(|f| f.number == number)
        .map(|f| f.bytes)
        .collect()
}

fn single(bytes: &[u8], number: u32) -> &[u8] {
    let found = fields(bytes, number);
    assert_eq!(found.len(), 1);
    found[0]
}

// Decode every reference, not prompt substring matching. Content addressing
// and nested result fields are checked independently of the builder.
fn identities(history: &PreparedHistory) -> Vec<(String, String)> {
    let mut pairs = Vec::new();
    for turn_id in fields(&history.state, 8) {
        let turn_blob = history.blobs.get(turn_id).unwrap();
        assert_eq!(Sha256::digest(turn_blob).as_slice(), turn_id);
        let turn = single(turn_blob, 1);
        let user_id = single(turn, 1);
        let user = history.blobs.get(user_id).unwrap();
        assert!(!single(user, 1).is_empty());
        for step_id in fields(turn, 2) {
            let step = history.blobs.get(step_id).unwrap();
            let calls = fields(step, 2);
            if calls.is_empty() {
                continue;
            }
            let mcp = single(calls[0], 15);
            let args = single(mcp, 1);
            let id = String::from_utf8(single(args, 3).to_vec()).unwrap();
            assert_eq!(single(args, 1), b"Read");
            let entry = single(args, 2);
            assert_eq!(single(entry, 1), b"path");
            // google.protobuf.Value.string_value = 3 inside map bytes.
            assert_eq!(single(single(entry, 2), 3), b"a.txt");
            let success = single(single(mcp, 2), 1);
            let content = single(single(single(success, 1), 1), 1);
            pairs.push((id, String::from_utf8(content.to_vec()).unwrap()));
        }
    }
    pairs
}

#[test]
fn cursor_history_identity_ordinary_preserves_ids() {
    let history = prepare(&fixture(), "composer-2.5").unwrap().unwrap();
    assert_eq!(
        identities(&history),
        [("authentic-17".into(), "file contents".into())]
    );
    let roots = fields(&history.state, 1);
    assert_eq!(roots.len(), 2);
    let root: Value = serde_json::from_slice(history.blobs.get(roots[0]).unwrap()).unwrap();
    assert_eq!(root["content"][0]["text"], "Read the file");
    let result: Value = serde_json::from_slice(history.blobs.get(roots[1]).unwrap()).unwrap();
    assert_eq!(result["role"], "assistant");
    assert_eq!(
        result["content"][0]["text"],
        "Tool output for Read (call_id: authentic-17, is_error: false):\nfile contents"
    );
}

#[test]
fn cursor_history_identity_compacted_preserves_ids() {
    let mut req = fixture();
    let before = prepare(&req, "composer-2.5").unwrap().unwrap();
    req["messages"][0]["content"] = json!("Summary retaining the pending call context");
    let after = prepare(&req, "composer-2.5").unwrap().unwrap();
    assert_eq!(before.conversation_id, after.conversation_id);
    assert_eq!(identities(&before), identities(&after));
}

#[test]
fn cursor_history_identity_recovered_preserves_ids() {
    let req = fixture();
    let before = prepare(&req, "composer-2.5").unwrap().unwrap();
    let id = before.conversation_id.clone();
    let pairs = identities(&before);
    drop(before); // Recovery reconstructs solely from client-retained history.
    let after = prepare(&req, "composer-2.5").unwrap().unwrap();
    assert_eq!(id, after.conversation_id);
    assert_eq!(pairs, identities(&after));
}

#[test]
fn cursor_history_identity_multi_round_preserves_ids() {
    let mut req = fixture();
    let mut next = req["messages"].as_array().unwrap()[1..].to_vec();
    next[0]["content"][0]["id"] = json!("authentic-18");
    next[1]["content"][0]["tool_use_id"] = json!("authentic-18");
    req["messages"].as_array_mut().unwrap().extend(next);
    let history = prepare(&req, "composer-2.5").unwrap().unwrap();
    assert_eq!(
        identities(&history),
        [
            ("authentic-17".into(), "file contents".into()),
            ("authentic-18".into(), "file contents".into())
        ]
    );
}

#[test]
fn cursor_continuation_guard_rejects_unpaired_tool_use() {
    let mut req = fixture();
    req["messages"].as_array_mut().unwrap().pop();
    assert!(prepare(&req, "composer-2.5").is_err());
}
#[test]
fn cursor_continuation_guard_rejects_unpaired_tool_result() {
    let mut req = fixture();
    req["messages"][2]["content"][0]["tool_use_id"] = json!("wrong");
    assert!(prepare(&req, "composer-2.5").is_err());
}
#[test]
fn cursor_continuation_guard_rejects_opaque_block() {
    let req = json!({"messages":[{"role":"user","content":[{"type":"mystery_block"}]}]});
    assert!(prepare(&req, "composer-2.5").is_err());
}
#[test]
fn cursor_continuation_guard_rejects_missing_tool_use_id() {
    let mut req = fixture();
    req["messages"][1]["content"][0]
        .as_object_mut()
        .unwrap()
        .remove("id");
    assert!(prepare(&req, "composer-2.5").is_err());
}
#[test]
fn cursor_continuation_guard_rejects_duplicates_order_and_missing_session() {
    let mut req = fixture();
    req.as_object_mut().unwrap().remove("metadata");
    assert!(prepare(&req, "composer-2.5").is_err());
    let mut req = fixture();
    req["messages"].as_array_mut().unwrap().swap(1, 2);
    assert!(prepare(&req, "composer-2.5").is_err());
    let mut req = fixture();
    let duplicate = req["messages"][2].clone();
    req["messages"].as_array_mut().unwrap().push(duplicate);
    assert!(prepare(&req, "composer-2.5").is_err());
    assert!(prepare(&fixture(), "grok-4.6").is_err());
    let mut req = fixture();
    req["checkpoint"] = json!("opaque");
    assert!(prepare(&req, "composer-2.5").is_err());
}
