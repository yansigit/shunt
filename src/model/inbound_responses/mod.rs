//! Translation adapters for inbound OpenAI Responses requests.
//!
//! `request`, `response`, and `collaboration` serve the strict local Anthropic
//! bridge. The upstream `messages_*`, `chat_*`, `events`, and `reasoning`
//! modules are translation core only; their endpoint dispatch remains deferred.

pub mod chat_request;
pub mod chat_stream;
pub mod collaboration;
pub mod events;
pub mod messages_request;
pub mod messages_stream;
pub mod reasoning;
pub mod request;
pub mod response;
