//! Request-local bounded Cursor KV/blob exchange (12-03, CUR-03/D-03).
//!
//! Wire evidence (OpenCodex rev 055c3ecf0de6c35f59195fc434d6b08525182b7f,
//! inspected read-only 2026-09-07; no state loaded, nothing executed there):
//! AgentServerMessage.kv_server_message = 4,
//! AgentClientMessage.kv_client_message = 3, KvServerMessage with id = 1,
//! get_blob_args = 2, set_blob_args = 3, GetBlobArgs.blob_id = 1,
//! SetBlobArgs with blob_id = 1 and blob_data = 2, KvClientMessage with
//! id = 1, get_blob_result = 2, set_blob_result = 3,
//! GetBlobResult.blob_data = 1 (optional), SetBlobResult.error = 1
//! (optional), Error.message = 1. storeCursorBlob keys blobs by raw SHA-256
//! of the bytes; a get for an unknown id replies with an empty result;
//! conversationTurns and toolCallStep store paired call/result steps as
//! content-addressed blobs referenced by id.
//!
//! Bounds below are gateway-side conservative choices, not observed wire
//! values: the 192-entry cap mirrors CURSOR_EXTERNAL_ROOT_BLOB_LIMIT (192)
//! from the same evidence; byte caps keep one request from growing the
//! process. The store is owned by a single active turn and dropped on
//! completion, error, or cancellation: no disk writeback, no cross-request
//! cache, no credential changes.
//!
//! `history` seeds this store with content-addressed conversation blobs.
//! `agent` owns it for the bidirectional request lifetime and answers KV
//! requests through the same bounded request-body channel.

use std::collections::HashMap;

use bytes::Bytes;

/// Maximum blob entries held for one request. Mirrors the 192-root replay
/// budget from the evidence; beyond it the turn is rejected, never pruned
/// silently (pruning would drop history the model still references).
pub const MAX_BLOB_ENTRIES: usize = 192;
/// Maximum bytes accepted for a single blob (server- or client-provided).
pub const MAX_BLOB_ENTRY_BYTES: usize = 512 * 1024;
/// Maximum total blob bytes held for one request.
pub const MAX_BLOB_TOTAL_BYTES: usize = 8 * 1024 * 1024;
/// Maximum opaque key size. Locally generated keys are SHA-256 digests;
/// incoming keys are opaque and bounded, regardless of their length.
pub const MAX_BLOB_ID_BYTES: usize = 128;

/// Bounded blobs owned by one active turn. Dropped with the turn.
#[derive(Debug, Default)]
pub struct RequestBlobStore {
    entries: HashMap<Vec<u8>, Vec<u8>>,
    total_bytes: usize,
    limits: BlobLimits,
}

/// Test-overridable copy of the module bounds. Production always uses
/// BlobLimits::default; tests shrink limits to prove rejection.
#[derive(Debug, Clone, Copy)]
pub struct BlobLimits {
    pub max_entries: usize,
    pub max_entry_bytes: usize,
    pub max_total_bytes: usize,
}

impl Default for BlobLimits {
    fn default() -> Self {
        Self {
            max_entries: MAX_BLOB_ENTRIES,
            max_entry_bytes: MAX_BLOB_ENTRY_BYTES,
            max_total_bytes: MAX_BLOB_TOTAL_BYTES,
        }
    }
}

/// Why a blob was refused. Surfaced explicitly; never a silent drop.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BlobReject {
    InvalidIdentity,
    /// A single blob exceeds the per-entry byte ceiling.
    EntryTooLarge {
        bytes: usize,
        max: usize,
    },
    /// The per-request entry count is exhausted.
    TooManyEntries {
        max: usize,
    },
    /// The per-request total byte budget is exhausted.
    CapacityExhausted,
}

impl std::fmt::Display for BlobReject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidIdentity => write!(
                f,
                "cursor blob identity is invalid or conflicts with stored content"
            ),
            Self::EntryTooLarge { bytes, max } => write!(
                f,
                "cursor blob of {bytes} bytes exceeds the per-request entry limit of {max} bytes"
            ),
            Self::TooManyEntries { max } => write!(
                f,
                "cursor blob store is full ({max} entries for one request)"
            ),
            Self::CapacityExhausted => write!(
                f,
                "cursor blob store byte budget for one request is exhausted"
            ),
        }
    }
}

/// A malformed KV frame on an otherwise intact stream. The turn fails
/// explicitly rather than answering a request it cannot parse.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KvWireError(pub String);

impl std::fmt::Display for KvWireError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "cursor: malformed KV frame: {}", self.0)
    }
}

impl RequestBlobStore {
    /// Empty store with production bounds.
    pub fn new() -> Self {
        Self::default()
    }

    /// Empty store with explicit bounds (tests only).
    #[cfg(test)]
    pub fn with_limits(limits: BlobLimits) -> Self {
        Self {
            entries: HashMap::new(),
            total_bytes: 0,
            limits,
        }
    }

    /// Number of blobs currently held.
    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether the store holds no blobs.
    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Fetch blob bytes by raw id. None means unknown: the caller replies
    /// with an empty result, mirroring getBlob miss semantics.
    pub fn get(&self, id: &[u8]) -> Option<&[u8]> {
        self.entries.get(id).map(Vec::as_slice)
    }

    /// Store bytes under an explicit id (server-minted or pre-registered
    /// history ids, which are not necessarily content-addressed).
    /// Re-storing an existing id is a no-op success.
    pub fn store_bytes_with_id(&mut self, id: &[u8], data: &[u8]) -> Result<(), BlobReject> {
        if id.is_empty()
            || id.len() > MAX_BLOB_ID_BYTES
            || self.entries.get(id).is_some_and(|old| old != data)
        {
            return Err(BlobReject::InvalidIdentity);
        }
        if data.len() > self.limits.max_entry_bytes {
            return Err(BlobReject::EntryTooLarge {
                bytes: data.len(),
                max: self.limits.max_entry_bytes,
            });
        }
        if !self.entries.contains_key(id) {
            if self.entries.len() >= self.limits.max_entries {
                return Err(BlobReject::TooManyEntries {
                    max: self.limits.max_entries,
                });
            }
            if self.total_bytes.saturating_add(data.len()) > self.limits.max_total_bytes {
                return Err(BlobReject::CapacityExhausted);
            }
            self.total_bytes += data.len();
            self.entries.insert(id.to_vec(), data.to_vec());
        }
        Ok(())
    }

    /// Store bytes under their SHA-256 digest and return the id, mirroring
    /// storeCursorBlob. The id IS the digest, so serve-time integrity holds
    /// by construction for ids minted here.
    #[allow(dead_code)]
    pub fn store_content(&mut self, data: &[u8]) -> Result<[u8; 32], BlobReject> {
        use sha2::Digest;
        let digest = sha2::Sha256::digest(data);
        let mut id = [0u8; 32];
        id.copy_from_slice(&digest);
        self.store_bytes_with_id(&id, data)?;
        Ok(id)
    }
}

/// A parsed server KV request: KvServerMessage with id plus get or set.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KvServerRequest {
    /// get_blob_args: fetch blob_id, reply with its bytes or empty.
    Get { id: u32, blob_id: Vec<u8> },
    /// set_blob_args: hold blob_data under blob_id.
    Set {
        id: u32,
        blob_id: Vec<u8>,
        blob_data: Vec<u8>,
    },
}

/// Answer a response payload that may carry AgentServerMessage field 4
/// (kv_server_message). Returns Ok(None) for non-KV payloads so the caller
/// continues normal extraction, Ok(Some(frame)) with a Connect-framed
/// AgentClientMessage field 3 (kv_client_message) reply for KV payloads,
/// or Err for a malformed KV frame.
pub fn handle_kv_payload(
    payload: &[u8],
    store: &mut RequestBlobStore,
) -> Result<Option<Bytes>, KvWireError> {
    let Some(kv_bytes) = checked_bytes_field(payload, 4, "kv_server_message")? else {
        return Ok(None);
    };
    let request = parse_kv_server(kv_bytes)?;
    let reply = match request {
        KvServerRequest::Get { id, blob_id } => {
            let result_inner = match store.get(&blob_id) {
                Some(data) => field_ld(1, data),
                // Unknown id: empty result, never an invented blob.
                None => Vec::new(),
            };
            encode_kv_client(id, &field_ld(2, &result_inner))
        }
        KvServerRequest::Set {
            id,
            blob_id,
            blob_data,
        } => {
            let result_inner = match store.store_bytes_with_id(&blob_id, &blob_data) {
                Ok(()) => Vec::new(),
                // Capacity refusal is explicit on the wire, not a silent drop.
                Err(reject) => field_ld(1, &field_str(1, &reject.to_string())),
            };
            encode_kv_client(id, &field_ld(3, &result_inner))
        }
    };
    Ok(Some(encode_connect_frame(&reply)))
}

/// Parse the inner KvServerMessage bytes. Err on any structural violation:
/// missing or duplicate bodies, oversized ids. Proto3 scalar id defaults to 0.
fn parse_kv_server(kv_bytes: &[u8]) -> Result<KvServerRequest, KvWireError> {
    validate_wire(kv_bytes)?;
    let mut id: Option<u32> = None;
    let mut get: Option<&[u8]> = None;
    let mut set: Option<&[u8]> = None;
    for field in iter_fields(kv_bytes) {
        match (field.field, field.wire) {
            (1, 0) => {
                if id.is_some() {
                    return Err(KvWireError("duplicate KvServerMessage.id".to_string()));
                }
                id = Some(
                    u32::try_from(field.varint)
                        .map_err(|_| KvWireError("KV id overflow".into()))?,
                );
            }
            (2, 2) => {
                if get.is_some() {
                    return Err(KvWireError("duplicate get_blob_args".to_string()));
                }
                get = Some(field.data);
            }
            (3, 2) => {
                if set.is_some() {
                    return Err(KvWireError("duplicate set_blob_args".to_string()));
                }
                set = Some(field.data);
            }
            (1..=3, _) => return Err(KvWireError("KV field has wrong wire encoding".into())),
            _ => {}
        }
    }
    let id = id.unwrap_or(0);
    match (get, set) {
        (Some(get_bytes), None) => {
            let blob_id = required_bytes_field(get_bytes, 1, "GetBlobArgs.blob_id")?;
            check_id_len(blob_id.len())?;
            Ok(KvServerRequest::Get {
                id,
                blob_id: blob_id.to_vec(),
            })
        }
        (None, Some(set_bytes)) => {
            let blob_id = required_bytes_field(set_bytes, 1, "SetBlobArgs.blob_id")?;
            let blob_data =
                checked_bytes_field(set_bytes, 2, "SetBlobArgs.blob_data")?.unwrap_or_default();
            if blob_data.len() > MAX_BLOB_ENTRY_BYTES {
                return Err(KvWireError("blob exceeds per-entry byte limit".into()));
            }
            check_id_len(blob_id.len())?;
            Ok(KvServerRequest::Set {
                id,
                blob_id: blob_id.to_vec(),
                blob_data: blob_data.to_vec(),
            })
        }
        (None, None) => Err(KvWireError(
            "KvServerMessage has no get/set body".to_string(),
        )),
        (Some(_), Some(_)) => Err(KvWireError(
            "KvServerMessage carries both get and set bodies".to_string(),
        )),
    }
}

fn check_id_len(len: usize) -> Result<(), KvWireError> {
    if len == 0 || len > MAX_BLOB_ID_BYTES {
        return Err(KvWireError(format!(
            "blob id of {len} bytes is not a valid key"
        )));
    }
    Ok(())
}

fn required_bytes_field<'a>(
    buf: &'a [u8],
    field_no: u64,
    name: &str,
) -> Result<&'a [u8], KvWireError> {
    checked_bytes_field(buf, field_no, name)?
        .ok_or_else(|| KvWireError(format!("{name} is missing")))
}

fn checked_bytes_field<'a>(
    buf: &'a [u8],
    field_no: u64,
    name: &str,
) -> Result<Option<&'a [u8]>, KvWireError> {
    validate_wire(buf)?;
    let mut seen: Option<&'a [u8]> = None;
    for field in iter_fields(buf) {
        if field.field == field_no {
            if field.wire != 2 {
                return Err(KvWireError(format!("invalid encoding for {name}")));
            }
            if seen.is_some() {
                return Err(KvWireError(format!("duplicate {name}")));
            }
            seen = Some(field.data);
        }
    }
    Ok(seen)
}

fn validate_wire(buf: &[u8]) -> Result<(), KvWireError> {
    super::wire::fields(buf)
        .try_for_each(|field| field.map(|_| ()).map_err(|e| KvWireError(e.to_string())))
}

/// Wrap a KvClientMessage body as AgentClientMessage field 3
/// (kv_client_message).
fn encode_kv_client(id: u32, result_field: &[u8]) -> Vec<u8> {
    let mut inner = field_varint(1, u64::from(id));
    inner.extend_from_slice(result_field);
    field_ld(3, &inner)
}

/// Decode a framed AgentClientMessage reply back to (kv id, result field
/// number, result bytes) for roundtrip assertions. Test-only.
#[cfg(test)]
fn decode_kv_client_reply(frame: &[u8]) -> Option<(u32, u64, Vec<u8>)> {
    let payload = split_connect_frame(frame)?;
    let kv_bytes = find_field(payload, 3)?;
    let mut id: Option<u32> = None;
    let mut result: Option<(u64, Vec<u8>)> = None;
    for field in iter_fields(kv_bytes) {
        match (field.field, field.wire) {
            (1, 0) => id = Some(field.varint as u32),
            (2, 2) | (3, 2) => {
                if result.is_some() {
                    return None;
                }
                result = Some((field.field, field.data.to_vec()));
            }
            _ => {}
        }
    }
    let (number, bytes) = result?;
    Some((id?, number, bytes))
}

// ---------------------------------------------------------------------------
// Minimal protobuf and Connect framing (hand-rolled to match the captured
// wire; same shapes as the private helpers in agent.rs, kept local so this
// module stays self-contained and unit-testable).
// ---------------------------------------------------------------------------

struct PbField<'a> {
    field: u64,
    wire: u8,
    data: &'a [u8],
    varint: u64,
}

fn iter_fields(buf: &[u8]) -> impl Iterator<Item = PbField<'_>> {
    // Production callers validate every byte before projecting fields.
    super::wire::fields(buf)
        .filter_map(Result::ok)
        .map(|field| PbField {
            field: u64::from(field.number),
            wire: field.wire,
            data: field.bytes,
            varint: field.varint,
        })
}

/// First length-delimited field with this number, or None.
#[cfg(test)]
fn find_field(buf: &[u8], field_no: u64) -> Option<&[u8]> {
    iter_fields(buf)
        .find(|f| f.field == field_no && f.wire == 2)
        .map(|f| f.data)
}

fn encode_varint(mut value: u64, out: &mut Vec<u8>) {
    while value >= 0x80 {
        out.push(((value as u8) & 0x7f) | 0x80);
        value >>= 7;
    }
    out.push(value as u8);
}

fn field_ld(field: u64, data: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(data.len() + 4);
    encode_varint((field << 3) | 2, &mut out);
    encode_varint(data.len() as u64, &mut out);
    out.extend_from_slice(data);
    out
}

fn field_str(field: u64, s: &str) -> Vec<u8> {
    field_ld(field, s.as_bytes())
}

fn field_varint(field: u64, value: u64) -> Vec<u8> {
    let mut out = Vec::new();
    encode_varint(field << 3, &mut out);
    encode_varint(value, &mut out);
    out
}

fn encode_connect_frame(payload: &[u8]) -> Bytes {
    crate::adapters::cursor::connect::encode_connect_frame(payload, 0)
}

#[cfg(test)]
fn split_connect_frame(frame: &[u8]) -> Option<&[u8]> {
    if frame.len() < 5 {
        return None;
    }
    let len = u32::from_be_bytes([frame[1], frame[2], frame[3], frame[4]]) as usize;
    if frame.len() != 5 + len {
        return None;
    }
    Some(&frame[5..])
}

#[cfg(test)]
#[path = "kv_tests.rs"]
mod tests;
