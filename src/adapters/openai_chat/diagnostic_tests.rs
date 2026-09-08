use super::*;

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
