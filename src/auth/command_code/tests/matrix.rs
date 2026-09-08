use super::*;

async fn assert_shape(error: AdapterError) {
    assert_auth_error(&error);
    let bytes = axum::body::to_bytes(error.response.into_body(), 65536)
        .await
        .unwrap();
    let body: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(body["type"], "error");
    assert_eq!(body["error"]["type"], "authentication_error");
    assert!(!body.to_string().contains("synthetic-file-key"));
}

#[tokio::test]
async fn command_code_matrix_auth_shapes_file_invariance_and_byte_bounds() {
    let _lock = FILE_TEST_LOCK.lock().await;
    let dir = lifetime_dir("final-matrix");
    std::fs::create_dir(&dir).unwrap();
    let path = dir.join("auth.json");
    let error = resolve_sources(Err(std::env::VarError::NotPresent), Some(&path))
        .await
        .unwrap_err();
    assert_shape(error).await;
    assert_eq!(std::fs::read_dir(&dir).unwrap().count(), 0);
    for contents in [
        br#"{bad"#.as_slice(),
        br#"{"apiKey":""}"#.as_slice(),
        br#"{"apiKey":null}"#.as_slice(),
    ] {
        std::fs::write(&path, contents).unwrap();
        let state = capture_file_state(&path);
        let error = resolve_sources(Err(std::env::VarError::NotPresent), Some(&path))
            .await
            .unwrap_err();
        assert_shape(error).await;
        assert_file_unchanged(&path, &state);
    }
    std::fs::write(&path, br#"{"apiKey":"synthetic-file-key"}"#).unwrap();
    let state = capture_file_state(&path);
    for token in ["", "bad token", "bad\r\nheader"] {
        let before = FILE_READS.load(std::sync::atomic::Ordering::SeqCst);
        let error = resolve_sources(Ok(token.into()), Some(&path))
            .await
            .unwrap_err();
        assert_shape(error).await;
        assert_eq!(FILE_READS.load(std::sync::atomic::Ordering::SeqCst), before);
        assert_file_unchanged(&path, &state);
    }
    for size in [
        CLI_AUTH_FILE_MAX_BYTES - 1,
        CLI_AUTH_FILE_MAX_BYTES,
        CLI_AUTH_FILE_MAX_BYTES + 1,
    ] {
        let mut bytes = br#"{"apiKey":"synthetic-file-key"}"#.to_vec();
        bytes.resize(size, b' ');
        std::fs::write(&path, &bytes).unwrap();
        let state = capture_file_state(&path);
        let result = resolve_sources(Err(std::env::VarError::NotPresent), Some(&path)).await;
        assert_eq!(
            result.is_ok(),
            size <= CLI_AUTH_FILE_MAX_BYTES,
            "file bytes {size}"
        );
        if let Err(error) = result {
            assert_shape(error).await;
        }
        assert_file_unchanged(&path, &state);
    }
    std::fs::remove_file(path).unwrap();
    std::fs::remove_dir(dir).unwrap();
}
