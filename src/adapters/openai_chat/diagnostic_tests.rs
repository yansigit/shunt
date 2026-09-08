use super::*;

#[test]
fn bound_tool_argument_split_utf8_at_and_plus_one() {
    use super::sse::{Decoder, Item};
    use serde_json::json;
    for excess in [0, 1] {
        let mut machine = OpenAiChatSseMachine::new_for_upstream("m");
        let prefix = format!("{{\"v\":\"{}", "x".repeat(1024 * 1024 - 10 + excess));
        machine.process_chunk_checked(&json!({"choices":[{"delta":{"tool_calls":[{"index":0,"id":"a","function":{"name":"f","arguments":prefix}}]}}]})).unwrap();
        let payload = json!({"choices":[{"delta":{"tool_calls":[{"index":0,"function":{"arguments":"é\"}"}}]}}]});
        let frame = format!("data: {payload}\n\n").into_bytes();
        let split = frame.iter().position(|byte| *byte == 0xc3).unwrap() + 1;
        let mut decoder = Decoder::default();
        assert!(decoder.push_one(&frame[..split]).unwrap().1.is_none());
        let Item::Json(value) = decoder.push_one(&frame[split..]).unwrap().1.unwrap() else {
            panic!("expected tool delta")
        };
        let result = machine.process_chunk_checked(&value);
        if excess == 0 {
            result.unwrap();
            machine
                .process_chunk_checked(
                    &json!({"choices":[{"delta":{},"finish_reason":"tool_calls"}]}),
                )
                .unwrap();
            machine.transport_close_checked().unwrap();
            assert_eq!(
                machine.final_json_checked().unwrap()["content"][0]["input"]["v"]
                    .as_str()
                    .unwrap()
                    .len(),
                1024 * 1024 - 8
            );
        } else {
            assert!(
                result.is_err(),
                "split multibyte fragment escaped the argument bound"
            );
        }
    }
}

#[test]
fn openai_chat_auth_classification_is_connect_only() {
    assert_eq!(
        OPENAI_CHAT_RETRY_SAFETY,
        crate::retry::RetrySafety::ConnectOnly
    );
    for status in [
        StatusCode::TOO_MANY_REQUESTS,
        StatusCode::BAD_GATEWAY,
        StatusCode::TEMPORARY_REDIRECT,
    ] {
        assert!(map_openai_chat_error(status, "").failure.is_none());
    }
}

#[test]
fn response_request_id_header_allowlist_and_bounds() {
    for name in [
        "x-request-id",
        "request-id",
        "authorization",
        "internal-trace-id",
    ] {
        for value in [
            "a".repeat(127),
            "a".repeat(128),
            "a".repeat(129),
            "".into(),
            "bad id".into(),
            "req_42-abc:def.1".into(),
        ] {
            let mut headers = HeaderMap::new();
            headers.insert(
                axum::http::HeaderName::from_bytes(name.as_bytes()).unwrap(),
                value.parse().unwrap(),
            );
            let selected = request_id_headers(&headers);
            let accepted = REQUEST_ID_HEADERS.contains(&name)
                && !value.is_empty()
                && value.len() <= 128
                && !value.contains(' ');
            assert_eq!(selected.contains_key(name), accepted, "{name} {value:?}");
        }
    }
}

#[tokio::test]
async fn response_non_success_diagnostics_are_provider_neutral() {
    for status in [StatusCode::UNAUTHORIZED, StatusCode::BAD_GATEWAY] {
        let error = map_openai_chat_error(status, r#"{"error":{"message":"secret-marker"}}"#);
        assert!(!error.message.contains("secret-marker"));
        let bytes = axum::body::to_bytes(error.response.into_body(), 4096)
            .await
            .unwrap();
        assert!(!std::str::from_utf8(&bytes)
            .unwrap()
            .contains("secret-marker"));
    }
}
