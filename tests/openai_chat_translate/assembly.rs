use serde_json::{json, Value};
use shunt::model::openai_chat_response::OpenAiChatSseMachine;

fn delta(machine: &mut OpenAiChatSseMachine, calls: Value) -> Result<(), String> {
    machine.process_chunk_checked(&json!({"choices":[{"delta":{"tool_calls":calls}}]})).map(|_| ()).map_err(|e| e.to_string())
}

fn finish(mut machine: OpenAiChatSseMachine) -> Result<Value, String> {
    machine.process_chunk_checked(&json!({"choices":[{"delta":{},"finish_reason":"tool_calls"}]})).map_err(|e| e.to_string())?;
    machine.transport_close_checked().map_err(|e| e.to_string())?;
    machine.final_json_checked().map_err(|e| e.to_string())
}

#[test]
fn assembly_identity_deferral_and_single_boundary_parse() {
    let mut machine = OpenAiChatSseMachine::new_for_upstream("m");
    delta(&mut machine, json!([{"index":7}])).unwrap();
    delta(&mut machine, json!([{"index":7,"function":{"arguments":"{\"v\":"}}])).unwrap();
    delta(&mut machine, json!([{"index":7,"id":"a","type":"function","function":{"name":"f","arguments":"1}"}}])).unwrap();
    assert_eq!(finish(machine).unwrap()["content"],json!([{"type":"tool_use","id":"a","name":"f","input":{"v":1}}]));
}

#[test]
fn assembly_interleave_preserves_first_arrival_order() {
    let mut machine = OpenAiChatSseMachine::new_for_upstream("m");
    for (index,id,args) in [(8,"a","{\"v\":"),(2,"b","{\"v\":"),(8,"a","1}"),(2,"b","2}")] {
        delta(&mut machine,json!([{"index":index,"id":id,"function":{"name":"f","arguments":args}}])).unwrap();
    }
    let out = finish(machine).unwrap();
    assert_eq!(out["content"].as_array().unwrap().len(),2);
    assert_eq!(out["content"][0]["id"],"a");
    assert_eq!(out["content"][0]["input"]["v"],1);
    assert_eq!(out["content"][1]["id"],"b");
    assert_eq!(out["content"][1]["input"]["v"],2);
}

#[test]
fn assembly_identity_conflict() {
    for changed in [json!({"index":0,"id":"b"}),json!({"index":0,"function":{"name":"g"}}),json!({"index":1,"id":"a"})] {
        let mut machine = OpenAiChatSseMachine::new_for_upstream("m");
        delta(&mut machine,json!([{"index":0,"id":"a","function":{"name":"f","arguments":"{}"}}])).unwrap();
        assert!(delta(&mut machine,json!([changed])).is_err(),"accepted {changed}");
    }
}

#[test]
fn assembly_unnamed_or_incomplete_terminal() {
    for call in [json!({"index":0,"id":"a","function":{"arguments":"{}"}}),json!({"index":0,"function":{"name":"f","arguments":"{}"}}),json!({"index":0,"id":"a","function":{"name":"f","arguments":"{"}})] {
        let mut machine = OpenAiChatSseMachine::new_for_upstream("m");
        delta(&mut machine,json!([call])).unwrap();
        assert!(finish(machine).is_err());
    }
}

#[test]
fn assembly_malformed_fields_fail_closed() {
    for call in [json!({"id":"a"}),json!({"index":-1}),json!({"index":0,"id":false}),json!({"index":0,"function":{"arguments":{}}}),json!({"index":0,"function":null}),json!({"index":0,"type":"custom"})] {
        let mut machine = OpenAiChatSseMachine::new_for_upstream("m");
        assert!(delta(&mut machine,json!([call])).is_err(),"accepted {call}");
    }
}

fn count_probe(count: usize) -> Result<Value,String> {
    let mut machine = OpenAiChatSseMachine::new_for_upstream("m");
    for index in 0..count {
        delta(&mut machine,json!([{"index":index,"id":format!("call_{index}"),"function":{"name":"f","arguments":"{}"}}]))?;
    }
    finish(machine)
}

#[test]
fn bound_tool_count_minus_one() { assert_eq!(count_probe(127).unwrap()["content"].as_array().unwrap().len(),127); }
#[test]
fn bound_tool_count_at() { assert_eq!(count_probe(128).unwrap()["content"].as_array().unwrap().len(),128); }
#[test]
fn bound_tool_count_plus_one() { assert!(count_probe(129).is_err()); }

fn args_probe(size: usize) -> Result<Value,String> {
    let mut machine = OpenAiChatSseMachine::new_for_upstream("m");
    let args = format!("{{\"v\":\"{}\"}}","x".repeat(size-8));
    let split = args.len()/2;
    delta(&mut machine,json!([{"index":0,"id":"a","function":{"name":"f","arguments":&args[..split]}}]))?;
    delta(&mut machine,json!([{"index":0,"function":{"arguments":&args[split..]}}]))?;
    finish(machine)
}
#[test]
fn bound_args_minus_one() { assert_eq!(args_probe(1024*1024-1).unwrap()["content"][0]["input"]["v"].as_str().unwrap().len(),1024*1024-9); }
#[test]
fn bound_args_at() { assert_eq!(args_probe(1024*1024).unwrap()["content"][0]["input"]["v"].as_str().unwrap().len(),1024*1024-8); }
#[test]
fn bound_args_plus_one() { assert!(args_probe(1024*1024+1).is_err()); }

#[test]
fn assembly_concurrent_requests_are_isolated() {
    let threads: Vec<_> = (0..4).map(|i| std::thread::spawn(move || {
        let mut machine = OpenAiChatSseMachine::new_for_upstream("m");
        delta(&mut machine,json!([{"index":0,"id":format!("call_{i}"),"function":{"name":"f","arguments":format!("{{\"v\":{i}}}")}}])).unwrap();
        let out = finish(machine).unwrap();
        assert_eq!(out["content"][0]["id"],format!("call_{i}"));
        assert_eq!(out["content"][0]["input"]["v"],i);
    })).collect();
    for thread in threads { thread.join().unwrap(); }
}
