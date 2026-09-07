# Native Antigravity tool identities

Native OAuth tool IDs use `call_antigravity_v2_` followed by URL-safe unpadded
base64 JSON containing `upstream_signature`, `scope_tag`, and `call_tag`.
Signatures retain their upstream bytes; an omitted parallel-call signature is
represented by null, not invented. The first Gemini 3 call must be signed.

Domain-separated SHA-256 tags bind account/session and signature/name/canonical
arguments/conversation-wide call ordinal. Object key order is normalized; array
order is significant. Exact integral floats normalize to integers (including
negative zero), so a client replaying `1.0` as `1` retains the same identity.
Integer inputs are never rounded through floating point. Native history rejects legacy or mismatched IDs before
inference. Non-native Gemini retains its v1 codec and rejects native IDs.

Sessions derive from the private account fingerprint and canonical first user
turn, not the evolving full request. Identical openings within one account
share a session; this fallback cannot distinguish those conversations. No new
public conversation-ID field is introduced. IDs contain neither raw account
fingerprints nor opening prompts. Upstream signatures remain encoded, not
encrypted, and clients should treat tool IDs as opaque sensitive values.

These stateless, unkeyed tags are context checks, not authentication or proof
that an upstream emitted a signature. An attacker who recomputes consistent
tags cannot be detected. No signature cache, persistent signing key, or
credential writeback is introduced. This is the user-approved limitation of
Plan 11-04, superseding its unconditional invented-signature rejection claim.

Clients migrating old native tool history must start a new conversation.
