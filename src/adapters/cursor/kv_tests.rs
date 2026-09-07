use super::*;
fn get_payload(id: u32, blob_id: &[u8]) -> Vec<u8> {
    let get_args = field_ld(1, blob_id);
    let mut kv = field_varint(1, u64::from(id));
    kv.extend(field_ld(2, &get_args));
    field_ld(4, &kv)
}
fn set_payload(id: u32, blob_id: &[u8], blob_data: &[u8]) -> Vec<u8> {
    let mut set_args = field_ld(1, blob_id);
    set_args.extend(field_ld(2, blob_data));
    let mut kv = field_varint(1, u64::from(id));
    kv.extend(field_ld(3, &set_args));
    field_ld(4, &kv)
}
fn reply_parts(frame: &[u8]) -> (u32, u64, Vec<u8>) {
    decode_kv_client_reply(frame).expect("KV reply should decode")
}
#[test]
fn cursor_history_identity_kv_set_get_roundtrip() {
    let mut store = RequestBlobStore::new();
    let blob_id = b"turn-1-step-0";
    let blob_data = b"paired-step";
    let reply = handle_kv_payload(&set_payload(7, blob_id, blob_data), &mut store)
        .expect("set should parse")
        .expect("set should reply");
    let (id, number, inner) = reply_parts(&reply);
    assert_eq!((id, number), (7, 3));
    assert!(inner.is_empty());
    let reply = handle_kv_payload(&get_payload(8, blob_id), &mut store)
        .expect("get should parse")
        .expect("get should reply");
    let (id, number, inner) = reply_parts(&reply);
    assert_eq!((id, number), (8, 2));
    let data = find_field(&inner, 1).expect("get hit carries blob_data");
    assert_eq!(data, blob_data);
}

#[test]
fn cursor_history_identity_kv_proto3_default_id_and_empty_data() {
    let mut store = RequestBlobStore::new();
    // uint32 id and bytes blob_data are proto3 scalars, not optional presence.
    let set = field_ld(4, &field_ld(3, &field_ld(1, b"empty")));
    let reply = handle_kv_payload(&set, &mut store).unwrap().unwrap();
    assert_eq!(reply_parts(&reply).0, 0);
    assert_eq!(store.get(b"empty"), Some(b"".as_slice()));
    let get = field_ld(4, &field_ld(2, &field_ld(1, b"empty")));
    let reply = handle_kv_payload(&get, &mut store).unwrap().unwrap();
    assert_eq!(reply_parts(&reply).0, 0);
    assert_eq!(find_field(&reply_parts(&reply).2, 1), Some(b"".as_slice()));
}
#[test]
fn cursor_history_identity_kv_sha_content_addressed() {
    let mut store = RequestBlobStore::new();
    let data = b"paired call/result step bytes";
    let id = store.store_content(data).expect("content store should fit");
    use sha2::Digest;
    let expect = sha2::Sha256::digest(data);
    assert_eq!(&id[..], &expect[..]);
    assert_eq!(store.get(&id).unwrap(), data);
    let reply = handle_kv_payload(&get_payload(3, &id), &mut store)
        .expect("get should parse")
        .expect("get should reply");
    let (_, number, inner) = reply_parts(&reply);
    assert_eq!(number, 2);
    assert_eq!(find_field(&inner, 1).unwrap(), data);
}
#[test]
fn cursor_history_identity_kv_opaque_keys_and_conflicts() {
    let mut store = RequestBlobStore::new();
    let opaque = [7; 32];
    store.store_bytes_with_id(&opaque, b"server blob").unwrap();
    assert_eq!(store.get(&opaque), Some(b"server blob".as_slice()));
    assert_eq!(
        store.store_bytes_with_id(&opaque, b"different"),
        Err(BlobReject::InvalidIdentity)
    );
    let id = store.store_content(b"history").unwrap();
    assert_eq!(
        store.store_bytes_with_id(&id, b"corruption"),
        Err(BlobReject::InvalidIdentity)
    );
    assert_eq!(store.get(&id), Some(b"history".as_slice()));
}
#[test]
fn cursor_history_identity_kv_unknown_id_empty_result() {
    let mut store = RequestBlobStore::new();
    let reply = handle_kv_payload(&get_payload(11, b"no-such-blob"), &mut store)
        .expect("get should parse")
        .expect("get should reply");
    let (id, number, inner) = reply_parts(&reply);
    assert_eq!((id, number), (11, 2));
    assert!(inner.is_empty());
}
#[test]
fn cursor_history_identity_kv_isolated_per_request() {
    let mut first = RequestBlobStore::new();
    let mut second = RequestBlobStore::new();
    handle_kv_payload(&set_payload(1, b"k", b"v"), &mut first)
        .expect("set parses")
        .expect("set replies");
    let reply = handle_kv_payload(&get_payload(2, b"k"), &mut second)
        .expect("get parses")
        .expect("get replies");
    let (_, _, inner) = reply_parts(&reply);
    assert!(inner.is_empty());
    assert_eq!(first.get(b"k").unwrap(), b"v");
    assert!(second.get(b"k").is_none());
}
#[test]
fn cursor_history_identity_kv_non_kv_passthrough() {
    let mut store = RequestBlobStore::new();
    assert!(handle_kv_payload(b"", &mut store)
        .expect("empty parses")
        .is_none());
    let text = field_ld(1, &field_ld(1, &field_str(1, "hello")));
    assert!(handle_kv_payload(&text, &mut store)
        .expect("text parses")
        .is_none());
    assert!(store.is_empty());
}
#[test]
fn cursor_continuation_guard_kv_rejects_entry_and_budget_limits() {
    let mut tiny = RequestBlobStore::with_limits(BlobLimits {
        max_entries: 1,
        max_entry_bytes: 4,
        max_total_bytes: 4,
    });
    let reply = handle_kv_payload(&set_payload(1, b"a", b"12345"), &mut tiny)
        .expect("oversize set parses")
        .expect("oversize set replies with explicit error");
    let (_, number, inner) = reply_parts(&reply);
    assert_eq!(number, 3);
    let err = find_field(&inner, 1).expect("set refusal carries error");
    let msg = find_field(err, 1).expect("error carries message");
    assert!(std::str::from_utf8(msg).unwrap().contains("exceeds"));
    let reply = handle_kv_payload(&set_payload(2, b"a", b"ok"), &mut tiny)
        .expect("fit set parses")
        .expect("fit set replies");
    assert!(reply_parts(&reply).2.is_empty());
    let reply = handle_kv_payload(&set_payload(3, b"b", b"ok"), &mut tiny)
        .expect("second entry parses")
        .expect("second entry replies with explicit error");
    let (_, number, inner) = reply_parts(&reply);
    assert_eq!(number, 3);
    assert!(!inner.is_empty());
    let mut total = RequestBlobStore::with_limits(BlobLimits {
        max_entries: 16,
        max_entry_bytes: 16,
        max_total_bytes: 8,
    });
    handle_kv_payload(&set_payload(4, b"x", b"1234"), &mut total)
        .expect("first parses")
        .expect("first replies");
    let reply = handle_kv_payload(&set_payload(5, b"y", b"12345"), &mut total)
        .expect("second parses")
        .expect("second replies with explicit error");
    assert!(!reply_parts(&reply).2.is_empty());
}
#[test]
fn cursor_continuation_guard_kv_rejects_malformed_corpus() {
    let mut store = RequestBlobStore::new();
    let get_args = field_ld(1, b"k");
    let mut set_args = field_ld(1, b"k");
    set_args.extend(field_ld(2, b"v"));
    let mut both = field_varint(1, 1);
    both.extend(field_ld(2, &get_args));
    both.extend(field_ld(3, &set_args));
    let mut no_body = field_varint(1, 1);
    no_body.extend(field_ld(9, b"ignored-unknown"));
    let mut dup_id = field_varint(1, 1);
    dup_id.extend(field_varint(1, 2));
    dup_id.extend(field_ld(2, &get_args));
    let big = vec![b'k'; 129];
    let mut missing_blob: Vec<u8> = field_varint(1, 1);
    missing_blob.extend(field_ld(2, &field_ld(2, b"v")));
    let cases: Vec<(&str, Vec<u8>)> = vec![
        (
            "truncated tail",
            [get_payload(1, b"k"), vec![0x80]].concat(),
        ),
        ("wrong outer wire", field_varint(4, 1)),
        (
            "overflow id",
            field_ld(
                4,
                &[field_varint(1, u64::MAX), field_ld(2, &get_args)].concat(),
            ),
        ),
        ("get and set", field_ld(4, &both)),
        ("no body", field_ld(4, &no_body)),
        ("missing blob_id", field_ld(4, &missing_blob)),
        ("oversized id", get_payload(1, &big)),
        ("empty id", get_payload(1, b"")),
        ("duplicate id", field_ld(4, &dup_id)),
    ];
    for (name, payload) in cases {
        let err = handle_kv_payload(&payload, &mut store).expect_err(name);
        assert!(err.to_string().contains("malformed KV"));
    }
    assert!(store.is_empty());
}
