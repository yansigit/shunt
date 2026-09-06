//! Dedicated inbound OpenAI Responses -> Anthropic Messages translation.
//!
//! This direction intentionally does not reuse or mutate the mature
//! Anthropic-client -> Responses translator. Native Responses routes never enter
//! this module and therefore retain byte-for-byte passthrough behavior.

pub mod collaboration;
pub mod request;
pub mod response;
