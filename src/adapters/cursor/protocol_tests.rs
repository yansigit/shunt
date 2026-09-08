use super::*;
use crate::adapters::cursor::connect::encode_connect_frame;

fn gzip(bytes: &[u8]) -> Vec<u8> {
    use std::io::Write;
    let mut writer = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::fast());
    writer.write_all(bytes).unwrap();
    writer.finish().unwrap()
}

#[tokio::test]
async fn cursor_proto_wire_strict_rejects_nested_argument_coercions() {
    for value in [
        vec![],
        vec![0x1a, 1, 255],
        vec![0x11, 0],
        [field_varint(1, 0), field_str(3, "second")].concat(),
    ] {
        let mut args = field_str(1, "Read");
        args.extend(field_str(3, "tool_authentic"));
        let mut entry = field_str(1, "path");
        entry.extend(field_ld(2, &value));
        args.extend(field_ld(2, &entry));
        let payload = field_ld(2, &field_ld(11, &args));
        let events: Vec<_> =
            super::tests::turn_from_frames(encode_connect_frame(payload, 0).to_vec())
                .await
                .into_event_stream()
                .collect()
                .await;
        assert_eq!(events.len(), 1);
        assert!(events[0].is_err(), "{events:?}");
    }
}

#[tokio::test]
async fn cursor_proto_wire_strict_rejects_corpus_but_keeps_unknown_fields() {
    for (name, payload) in crate::adapters::cursor::connect::cursor_malformed_protobuf_corpus() {
        let events: Vec<_> =
            super::tests::turn_from_frames(encode_connect_frame(payload, 0).to_vec())
                .await
                .into_event_stream()
                .collect()
                .await;
        assert!(matches!(&events[..], [Err(_)]), "{name}: {events:?}");
    }
    let mut frames = encode_connect_frame(field_ld(99, b"opaque future data"), 0).to_vec();
    frames.extend(encode_connect_frame(b"{}", FLAG_END));
    let events: Vec<_> = super::tests::turn_from_frames(frames)
        .await
        .into_event_stream()
        .collect()
        .await;
    assert!(matches!(&events[..], [Ok(CursorStreamEvent::End)]));
}

#[tokio::test]
#[allow(clippy::await_holding_lock)] // Serialize the shared cross-runtime offload observer.
async fn cursor_framing_rejection_end_table_preserves_valid_gzip_and_auth() {
    let _observer = super::super::offload::OFFLOAD_OBSERVER
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    for (name, payload, flags, error) in [
        ("empty", vec![], FLAG_END, None),
        ("object", b"{}".to_vec(), FLAG_END, None),
        ("gzip object", gzip(b"{}"), FLAG_END | FLAG_GZIP, None),
        (
            "truncated JSON",
            b"{\"error\":".to_vec(),
            FLAG_END,
            Some(502),
        ),
        ("nonobject", b"[]".to_vec(), FLAG_END, Some(502)),
        (
            "wrong error type",
            b"{\"error\":42}".to_vec(),
            FLAG_END,
            Some(502),
        ),
        (
            "corrupt gzip",
            b"not gzip".to_vec(),
            FLAG_END | FLAG_GZIP,
            Some(502),
        ),
        (
            "gzip auth",
            gzip(br#"{"error":{"code":"unauthenticated","message":"expired"}}"#),
            FLAG_END | FLAG_GZIP,
            Some(401),
        ),
        (
            "unsupported flags",
            b"{}".to_vec(),
            FLAG_END | 0x80,
            Some(502),
        ),
    ] {
        let events: Vec<_> =
            super::tests::turn_from_frames(encode_connect_frame(payload, flags).to_vec())
                .await
                .into_event_stream()
                .collect()
                .await;
        assert_eq!(events.len(), 1, "{name}: {events:?}");
        match error {
            None => assert!(
                matches!(events[0], Ok(CursorStreamEvent::End)),
                "{name}: {events:?}"
            ),
            Some(status) => assert!(
                matches!(&events[0], Err(e) if e.status == status),
                "{name}: {events:?}"
            ),
        }
    }
}
