use serde_json::{json, Value};
use shunt::model::command_code_response::{CommandCodeMachine, FailureKind};

fn finish(kind: &str, reason: &str) -> Value {
    json!({"type":kind,"finishReason":reason,"usage":{"inputTokens":10,"outputTokens":4}})
}

#[test]
fn command_code_translate_machine_terminal_grammar_and_first_usage() {
    for kind in ["finish-step", "finish"] {
        let mut m = CommandCodeMachine::new("alias");
        m.process_record_checked(&finish(kind, "stop")).unwrap();
        assert_eq!(m.final_ndjson_checked().unwrap()["stop_reason"], "end_turn");
    }
    let mut m = CommandCodeMachine::new("alias");
    m.process_record_checked(&finish("finish-step", "stop"))
        .unwrap();
    m.process_record_checked(&json!({"type":"finish","rawFinishReason":"stop","totalUsage":{"inputTokens":100,"outputTokens":200}})).unwrap();
    assert_eq!(
        m.final_ndjson_checked().unwrap()["usage"]["input_tokens"],
        10
    );
    for (first, second) in [
        (finish("finish", "stop"), finish("finish", "stop")),
        (finish("finish", "stop"), finish("finish-step", "stop")),
        (finish("finish-step", "stop"), finish("finish-step", "stop")),
        (finish("finish-step", "stop"), finish("finish", "error")),
        (
            finish("finish-step", "stop"),
            json!({"type":"text-delta","text":"trailing"}),
        ),
    ] {
        let mut m = CommandCodeMachine::new("alias");
        m.process_record_checked(&first).unwrap();
        assert!(m.process_record_checked(&second).is_err());
        assert!(m.final_ndjson_checked().is_err());
    }
    let mut m = CommandCodeMachine::new("alias");
    m.process_record_checked(&finish("finish-step", "stop"))
        .unwrap();
    m.process_record_checked(&finish("finish", "stop")).unwrap();
    assert!(m.process_record_checked(&finish("finish", "stop")).is_err());
    let mut m = CommandCodeMachine::new("alias");
    assert!(m
        .process_record_checked(
            &json!({"type":"finish","rawFinishReason":"stop","finishReason":"error"})
        )
        .is_err());
}

#[test]
fn command_code_translate_machine_usage_precision_and_consistency() {
    for usage in [
        json!(null),
        json!([]),
        json!({"inputTokens":-1}),
        json!({"outputTokens":0.1}),
        json!({"inputTokens":18446744073709551616.0}),
        json!({"inputTokens":u64::MAX,"outputTokens":1}),
        json!({"inputTokens":4,"outputTokens":2,"totalTokens":9}),
        json!({"inputTokenDetails":null}),
        json!({"inputTokens":10,"inputTokenDetails":{"cacheReadTokens":-1}}),
        json!({"inputTokens":10,"inputTokenDetails":{"cacheWriteTokens":0.1}}),
        json!({"inputTokens":10,"inputTokenDetails":{"cacheReadTokens":8,"cacheWriteTokens":3}}),
        json!({"inputTokens":u64::MAX,"inputTokenDetails":{"cacheReadTokens":u64::MAX,"cacheWriteTokens":1}}),
    ] {
        let mut m = CommandCodeMachine::new("alias");
        assert!(m
            .process_record_checked(&json!({"type":"finish","finishReason":"stop","usage":usage}))
            .is_err());
        assert!(m.final_ndjson_checked().is_err());
    }
    let mut m = CommandCodeMachine::new("alias");
    m.process_record_checked(&json!({"type":"finish","finishReason":"stop","usage":{"inputTokens":u64::MAX,"outputTokens":0}})).unwrap();
    assert_eq!(
        m.final_ndjson_checked().unwrap()["usage"]["input_tokens"],
        u64::MAX
    );
    let mut m = CommandCodeMachine::new("alias");
    m.process_record_checked(&finish("finish-step", "stop"))
        .unwrap();
    assert!(m
        .process_record_checked(
            &json!({"type":"finish","finishReason":"stop","usage":{"outputTokens":-1}})
        )
        .is_err());
}

#[test]
fn command_code_translate_machine_failed_finish_retains_usage_redacted() {
    let mut m = CommandCodeMachine::new("alias");
    let error = m.process_record_checked(&json!({"type":"finish","finishReason":"error",
        "usage":{"inputTokens":10,"outputTokens":4,"inputTokenDetails":{"cacheReadTokens":6,"cacheWriteTokens":2}}})).unwrap_err();
    assert_eq!(error.kind, FailureKind::Provider);
    assert_eq!(error.body()["usage"]["input_tokens"], 2);
    assert_eq!(error.body()["usage"]["output_tokens"], 4);
    assert!(m.final_ndjson_checked().is_err());
    let mut m = CommandCodeMachine::new("alias");
    let error = m
        .process_record_checked(
            &json!({"type":"error","error":{"message":"synthetic-private-upstream-detail"}}),
        )
        .unwrap_err();
    assert!(!error.body().to_string().contains("synthetic-private"));
    assert_eq!(error.kind, FailureKind::Provider);
}

#[test]
fn command_code_translate_machine_tool_first_order_and_no_stream_retention() {
    let mut m = CommandCodeMachine::new_streaming("alias");
    let events = m
        .process_record_checked(
            &json!({"type":"tool-call","toolCallId":"a","toolName":"lookup","args":"{\"x\":1}"}),
        )
        .unwrap();
    assert_eq!(
        events.iter().map(|e| e.event.as_str()).collect::<Vec<_>>(),
        [
            "message_start",
            "content_block_start",
            "content_block_delta",
            "content_block_stop"
        ]
    );
    assert_eq!(events[1].data["content_block"]["id"], "a");
    let events = m
        .process_record_checked(&json!({"type":"text-delta","text":"next"}))
        .unwrap();
    assert_eq!(events[0].event, "content_block_start");
    assert_eq!(m.retained_content_blocks(), 0);
    m.process_record_checked(&finish("finish", "tool_use"))
        .unwrap();
    let terminal = m.transport_close_checked().unwrap();
    assert_eq!(
        terminal
            .iter()
            .filter(|e| e.event == "message_stop")
            .count(),
        1
    );
    assert!(m.transport_close_checked().unwrap().is_empty());
    assert!(
        m.final_ndjson_checked().is_err(),
        "streaming never retains a unary copy"
    );
}

#[test]
fn command_code_translate_machine_invalid_records_and_arguments_are_sticky() {
    for record in [
        json!(null),
        json!({}),
        json!({"type":3}),
        json!({"type":"text-delta","text":null}),
        json!({"type":"tool-call","toolName":"x","input":{}}),
        json!({"type":"tool-call","toolCallId":"a","toolName":"x"}),
        json!({"type":"tool-call","toolCallId":"a","toolName":"x","input":"{\"unfinished\":"}),
        json!({"type":"tool-call","toolCallId":"a","toolName":"x","input":[]}),
        json!({"type":"tool-call","toolCallId":"a","toolName":"x","input":{},"args":{}}),
        json!({"type":"finish","finishReason":"mystery"}),
        json!({"type":"finish"}),
    ] {
        let mut m = CommandCodeMachine::new("alias");
        assert!(m.process_record_checked(&record).is_err());
        assert!(m.process_record_checked(&finish("finish", "stop")).is_err());
        assert!(m.final_ndjson_checked().is_err());
    }
    let mut m = CommandCodeMachine::new("alias");
    assert_eq!(
        m.process_record_checked(&json!({"type":"unknown"}))
            .unwrap_err()
            .kind,
        FailureKind::UnknownRecord
    );
    assert!(CommandCodeMachine::new("alias")
        .final_ndjson_checked()
        .is_err());
    let mut m = CommandCodeMachine::new("alias");
    let call = json!({"type":"tool-call","toolCallId":"a","toolName":"x","input":{}});
    m.process_record_checked(&call).unwrap();
    assert!(m.process_record_checked(&call).is_err());
}

#[test]
fn command_code_translate_machine_contiguous_text_and_reasoning_order() {
    let mut m = CommandCodeMachine::new("alias");
    for (kind, text) in [
        ("text-delta", "a"),
        ("text-delta", "é"),
        ("reasoning-delta", "why"),
        ("text-delta", "z"),
    ] {
        m.process_record_checked(&json!({"type":kind,"text":text}))
            .unwrap();
    }
    m.process_record_checked(&finish("finish", "length"))
        .unwrap();
    let result = m.final_ndjson_checked().unwrap();
    assert_eq!(
        result["content"],
        json!([{"type":"text","text":"aé"},{"type":"thinking","thinking":"why"},{"type":"text","text":"z"}])
    );
    assert_eq!(result["stop_reason"], "max_tokens");
}
