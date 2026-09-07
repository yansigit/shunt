use prost::Message;

use super::connect::encode_connect_frame;
use super::proto::{AgentServerMessage, InteractionUpdate, TextDelta, ThinkingDelta, TurnEnded};

pub(crate) fn text_frame(text: &str) -> Vec<u8> {
    encode_agent_message(AgentServerMessage {
        interaction_update: Some(InteractionUpdate {
            thinking_delta: None,
            text_delta: Some(TextDelta {
                text: text.to_string(),
            }),
            turn_ended: None,
        }),
        exec_server_message: None,
    })
}

pub(crate) fn thinking_frame(text: &str) -> Vec<u8> {
    encode_agent_message(AgentServerMessage {
        interaction_update: Some(InteractionUpdate {
            thinking_delta: Some(ThinkingDelta {
                text: text.to_string(),
            }),
            text_delta: None,
            turn_ended: None,
        }),
        exec_server_message: None,
    })
}

pub(crate) fn usage_frame(input: u64, output: u64) -> Vec<u8> {
    usage_frame_full(input, output, 0, 0)
}

pub(crate) fn usage_frame_full(
    input: u64,
    output: u64,
    cache_read: u64,
    cache_write: u64,
) -> Vec<u8> {
    encode_agent_message(AgentServerMessage {
        interaction_update: Some(InteractionUpdate {
            thinking_delta: None,
            text_delta: None,
            turn_ended: Some(TurnEnded {
                input_tokens: input,
                output_tokens: output,
                cache_read_tokens: cache_read,
                cache_write_tokens: cache_write,
            }),
        }),
        exec_server_message: None,
    })
}

pub(crate) fn end_frame() -> Vec<u8> {
    encode_connect_frame(b"", 2).to_vec()
}

fn encode_agent_message(msg: AgentServerMessage) -> Vec<u8> {
    let mut payload = Vec::new();
    msg.encode(&mut payload).unwrap();
    encode_connect_frame(&payload, 0).to_vec()
}

/// Active AgentService/Run fixtures, separate from the retired schema above.
/// Source: OpenCodex MIT, src/adapters/cursor/gen/agent_pb.ts at
/// 055c3ecf0de6c35f59195fc434d6b08525182b7f; schema checked 2026-09-07.
/// These test-local prost types verify encoding, not current production support.
pub(crate) mod active_wire {
    use super::*;

    #[derive(Clone, PartialEq, Message)]
    pub(crate) struct Envelope {
        #[prost(message, optional, tag = "1")]
        pub interaction: Option<Interaction>,
        #[prost(message, optional, tag = "3")]
        pub checkpoint: Option<Checkpoint>,
    }
    #[derive(Clone, PartialEq, Message)]
    pub(crate) struct Interaction {
        #[prost(message, optional, tag = "8")]
        pub token_delta: Option<TokenDelta>,
        #[prost(message, optional, tag = "14")]
        pub turn_ended: Option<Ended>,
    }
    #[derive(Clone, PartialEq, Message)]
    pub(crate) struct TokenDelta {
        #[prost(int32, tag = "1")]
        pub tokens: i32,
    }
    #[derive(Clone, PartialEq, Message)]
    pub(crate) struct Ended {}
    #[derive(Clone, PartialEq, Message)]
    pub(crate) struct Checkpoint {
        #[prost(message, optional, tag = "5")]
        pub token_details: Option<TokenDetails>,
    }
    #[derive(Clone, PartialEq, Message)]
    pub(crate) struct TokenDetails {
        #[prost(uint32, tag = "1")]
        pub used_tokens: u32,
    }
    pub(crate) fn delta(tokens: i32) -> Vec<u8> {
        Envelope {
            interaction: Some(Interaction {
                token_delta: Some(TokenDelta { tokens }),
                turn_ended: None,
            }),
            checkpoint: None,
        }
        .encode_to_vec()
    }
    pub(crate) fn checkpoint(used_tokens: u32) -> Vec<u8> {
        Envelope {
            interaction: None,
            checkpoint: Some(Checkpoint {
                token_details: Some(TokenDetails { used_tokens }),
            }),
        }
        .encode_to_vec()
    }
    pub(crate) fn ended() -> Vec<u8> {
        Envelope {
            interaction: Some(Interaction {
                token_delta: None,
                turn_ended: Some(Ended {}),
            }),
            checkpoint: None,
        }
        .encode_to_vec()
    }

    #[test]
    fn cursor_w0_usage_facts_output_delta_has_exact_tags() {
        let bytes = delta(42);
        assert_eq!(bytes, [0x0a, 4, 0x42, 2, 8, 42]);
        assert_eq!(
            Envelope::decode(bytes.as_slice())
                .unwrap()
                .interaction
                .unwrap()
                .token_delta
                .unwrap()
                .tokens,
            42
        );
    }
    #[test]
    fn cursor_w0_usage_facts_checkpoints_are_absolute_not_additive() {
        let mut context = None;
        let mut output = 0;
        for bytes in [checkpoint(10000), delta(42), checkpoint(10300)] {
            let event = Envelope::decode(bytes.as_slice()).unwrap();
            if let Some(value) = event.checkpoint {
                context = Some(value.token_details.unwrap().used_tokens);
            }
            if let Some(value) = event.interaction {
                output += value.token_delta.unwrap().tokens;
            }
        }
        assert_eq!(context, Some(10300));
        assert_eq!(output, 42);
        // Inferred input only; context occupancy is not actual billed input.
        let inferred_input = context.map(|n| n.saturating_sub(output as u32));
        assert_eq!(inferred_input, Some(10258));
    }
    #[test]
    fn cursor_w0_usage_facts_turn_ended_has_exact_tags() {
        assert_eq!(ended(), [0x0a, 2, 0x72, 0]);
        assert!(Envelope::decode(ended().as_slice())
            .unwrap()
            .interaction
            .unwrap()
            .turn_ended
            .is_some());
    }
    #[test]
    fn cursor_w0_usage_facts_absence_is_not_measured_zero() {
        let event = Envelope::decode(&[][..]).unwrap();
        assert!(event.interaction.is_none());
        assert!(event.checkpoint.is_none());
    }
    #[test]
    fn cursor_w0_usage_facts_negative_int32_is_valid_encoding_not_valid_usage() {
        let bytes = delta(-1);
        assert_eq!(bytes.len(), 15);
        assert_eq!(
            Envelope::decode(bytes.as_slice())
                .unwrap()
                .interaction
                .unwrap()
                .token_delta
                .unwrap()
                .tokens,
            -1
        );
    }
}
