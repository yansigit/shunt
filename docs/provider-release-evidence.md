# Provider Release Evidence Ledger

Machine-checked release ledger for the v2 provider compatibility surface
(REL-01/REL-02). One row per finite documented (provider, auth, exact model,
wire) tuple. Generic/account-catalog contracts are separate `unbound_contracts`,
not invented model identities. Named tests establish shared protocol/auth
behavior with synthetic fixtures, not per-model live availability.
Evidence classes stay separate: every row below is
source-derived (provenance class `source`); `capture` and `live` are distinct
row fields and remain `none` until Phase 16 live-smoke plans record actual
bounded results. No claim below is inferred from a passing full suite, host
model dispatch, or a live call. Scenarios are `covered` (named existing test
at file#test that exercises the scenario for this wire), `gap` (applicable
coverage the repository does not yet prove — recorded honestly, never
relabeled), or `not_applicable` (the scenario cannot occur on this wire, with
rationale).

```release-ledger
{
  "schema": "provider-release-evidence/1",
  "as_of": "2026-09-09",
  "rows": [
    {
      "provider": "anthropic",
      "auth": "passthrough",
      "model": "claude-fable-5",
      "model_policy": "Exact finite builtin discovery catalog (src/discovery.rs BUILTIN_MODELS); other client-supplied ids pass through without a support claim.",
      "wire": "https://api.anthropic.com/v1/messages",
      "scenarios": {
        "normal": {
          "status": "covered",
          "evidence": "tests/passthrough.rs#messages_forwards_anthropic_headers_verbatim_and_preserves_query"
        },
        "stream": {
          "status": "covered",
          "evidence": "tests/passthrough.rs#relays_sse_response_with_content_type_preserved"
        },
        "tools": {
          "status": "covered",
          "evidence": "tests/inbound_anthropic_translation.rs#enabled_collaboration_translates_and_restores_json_tool_call"
        },
        "terminal": {
          "status": "covered",
          "evidence": "tests/inbound_anthropic_translation.rs#response_premature_stream_eof_is_failed_not_completed"
        },
        "malformed": {
          "status": "covered",
          "evidence": "tests/passthrough.rs#messages_rejects_duplicate_top_level_model_fields"
        },
        "truncation": {
          "status": "covered",
          "evidence": "tests/failover.rs#redispatch_gate_client_visible_truncated_body_stops_without_replaying_turn"
        },
        "auth": {
          "status": "covered",
          "evidence": "tests/passthrough.rs#messages_forwards_incoming_credentials_unchanged"
        },
        "cancellation": {
          "status": "covered",
          "evidence": "tests/passthrough.rs#response_drop_releases_upstream_work_and_global_capacity"
        },
        "retry": {
          "status": "covered",
          "evidence": "tests/retry.rs#messages_path_does_not_retry_transient_status_despite_enabled_policy"
        }
      },
      "provenance": {
        "class": "source",
        "repository": "shunt",
        "revision": "4b7b5bae1956a304b0ac5359d8dd61124c6728dc",
        "inspected": "2026-09-09",
        "sanitization": "sanitized: no credentials, account identifiers, or private content"
      },
      "capture": "none",
      "live": "none",
      "evidence_scope": "Named tests exercise the shared adapter/auth contract and synthetic fixtures, not live service availability or entitlement for this model. Cancellation includes terminal-failure or process-shutdown cleanup where named; client-disconnect coverage is separately identifiable from the cited test."
    },
    {
      "provider": "anthropic",
      "auth": "passthrough",
      "model": "claude-haiku-4-5-20251001",
      "model_policy": "Exact finite builtin discovery catalog (src/discovery.rs BUILTIN_MODELS); other client-supplied ids pass through without a support claim.",
      "wire": "https://api.anthropic.com/v1/messages",
      "scenarios": {
        "normal": {
          "status": "covered",
          "evidence": "tests/passthrough.rs#messages_forwards_anthropic_headers_verbatim_and_preserves_query"
        },
        "stream": {
          "status": "covered",
          "evidence": "tests/passthrough.rs#relays_sse_response_with_content_type_preserved"
        },
        "tools": {
          "status": "covered",
          "evidence": "tests/inbound_anthropic_translation.rs#enabled_collaboration_translates_and_restores_json_tool_call"
        },
        "terminal": {
          "status": "covered",
          "evidence": "tests/inbound_anthropic_translation.rs#response_premature_stream_eof_is_failed_not_completed"
        },
        "malformed": {
          "status": "covered",
          "evidence": "tests/passthrough.rs#messages_rejects_duplicate_top_level_model_fields"
        },
        "truncation": {
          "status": "covered",
          "evidence": "tests/failover.rs#redispatch_gate_client_visible_truncated_body_stops_without_replaying_turn"
        },
        "auth": {
          "status": "covered",
          "evidence": "tests/passthrough.rs#messages_forwards_incoming_credentials_unchanged"
        },
        "cancellation": {
          "status": "covered",
          "evidence": "tests/passthrough.rs#response_drop_releases_upstream_work_and_global_capacity"
        },
        "retry": {
          "status": "covered",
          "evidence": "tests/retry.rs#messages_path_does_not_retry_transient_status_despite_enabled_policy"
        }
      },
      "provenance": {
        "class": "source",
        "repository": "shunt",
        "revision": "4b7b5bae1956a304b0ac5359d8dd61124c6728dc",
        "inspected": "2026-09-09",
        "sanitization": "sanitized: no credentials, account identifiers, or private content"
      },
      "capture": "none",
      "live": "none",
      "evidence_scope": "Named tests exercise the shared adapter/auth contract and synthetic fixtures, not live service availability or entitlement for this model. Cancellation includes terminal-failure or process-shutdown cleanup where named; client-disconnect coverage is separately identifiable from the cited test."
    },
    {
      "provider": "anthropic",
      "auth": "passthrough",
      "model": "claude-opus-4-1-20250805",
      "model_policy": "Exact finite builtin discovery catalog (src/discovery.rs BUILTIN_MODELS); other client-supplied ids pass through without a support claim.",
      "wire": "https://api.anthropic.com/v1/messages",
      "scenarios": {
        "normal": {
          "status": "covered",
          "evidence": "tests/passthrough.rs#messages_forwards_anthropic_headers_verbatim_and_preserves_query"
        },
        "stream": {
          "status": "covered",
          "evidence": "tests/passthrough.rs#relays_sse_response_with_content_type_preserved"
        },
        "tools": {
          "status": "covered",
          "evidence": "tests/inbound_anthropic_translation.rs#enabled_collaboration_translates_and_restores_json_tool_call"
        },
        "terminal": {
          "status": "covered",
          "evidence": "tests/inbound_anthropic_translation.rs#response_premature_stream_eof_is_failed_not_completed"
        },
        "malformed": {
          "status": "covered",
          "evidence": "tests/passthrough.rs#messages_rejects_duplicate_top_level_model_fields"
        },
        "truncation": {
          "status": "covered",
          "evidence": "tests/failover.rs#redispatch_gate_client_visible_truncated_body_stops_without_replaying_turn"
        },
        "auth": {
          "status": "covered",
          "evidence": "tests/passthrough.rs#messages_forwards_incoming_credentials_unchanged"
        },
        "cancellation": {
          "status": "covered",
          "evidence": "tests/passthrough.rs#response_drop_releases_upstream_work_and_global_capacity"
        },
        "retry": {
          "status": "covered",
          "evidence": "tests/retry.rs#messages_path_does_not_retry_transient_status_despite_enabled_policy"
        }
      },
      "provenance": {
        "class": "source",
        "repository": "shunt",
        "revision": "4b7b5bae1956a304b0ac5359d8dd61124c6728dc",
        "inspected": "2026-09-09",
        "sanitization": "sanitized: no credentials, account identifiers, or private content"
      },
      "capture": "none",
      "live": "none",
      "evidence_scope": "Named tests exercise the shared adapter/auth contract and synthetic fixtures, not live service availability or entitlement for this model. Cancellation includes terminal-failure or process-shutdown cleanup where named; client-disconnect coverage is separately identifiable from the cited test."
    },
    {
      "provider": "anthropic",
      "auth": "passthrough",
      "model": "claude-opus-4-5-20251101",
      "model_policy": "Exact finite builtin discovery catalog (src/discovery.rs BUILTIN_MODELS); other client-supplied ids pass through without a support claim.",
      "wire": "https://api.anthropic.com/v1/messages",
      "scenarios": {
        "normal": {
          "status": "covered",
          "evidence": "tests/passthrough.rs#messages_forwards_anthropic_headers_verbatim_and_preserves_query"
        },
        "stream": {
          "status": "covered",
          "evidence": "tests/passthrough.rs#relays_sse_response_with_content_type_preserved"
        },
        "tools": {
          "status": "covered",
          "evidence": "tests/inbound_anthropic_translation.rs#enabled_collaboration_translates_and_restores_json_tool_call"
        },
        "terminal": {
          "status": "covered",
          "evidence": "tests/inbound_anthropic_translation.rs#response_premature_stream_eof_is_failed_not_completed"
        },
        "malformed": {
          "status": "covered",
          "evidence": "tests/passthrough.rs#messages_rejects_duplicate_top_level_model_fields"
        },
        "truncation": {
          "status": "covered",
          "evidence": "tests/failover.rs#redispatch_gate_client_visible_truncated_body_stops_without_replaying_turn"
        },
        "auth": {
          "status": "covered",
          "evidence": "tests/passthrough.rs#messages_forwards_incoming_credentials_unchanged"
        },
        "cancellation": {
          "status": "covered",
          "evidence": "tests/passthrough.rs#response_drop_releases_upstream_work_and_global_capacity"
        },
        "retry": {
          "status": "covered",
          "evidence": "tests/retry.rs#messages_path_does_not_retry_transient_status_despite_enabled_policy"
        }
      },
      "provenance": {
        "class": "source",
        "repository": "shunt",
        "revision": "4b7b5bae1956a304b0ac5359d8dd61124c6728dc",
        "inspected": "2026-09-09",
        "sanitization": "sanitized: no credentials, account identifiers, or private content"
      },
      "capture": "none",
      "live": "none",
      "evidence_scope": "Named tests exercise the shared adapter/auth contract and synthetic fixtures, not live service availability or entitlement for this model. Cancellation includes terminal-failure or process-shutdown cleanup where named; client-disconnect coverage is separately identifiable from the cited test."
    },
    {
      "provider": "anthropic",
      "auth": "passthrough",
      "model": "claude-opus-4-6",
      "model_policy": "Exact finite builtin discovery catalog (src/discovery.rs BUILTIN_MODELS); other client-supplied ids pass through without a support claim.",
      "wire": "https://api.anthropic.com/v1/messages",
      "scenarios": {
        "normal": {
          "status": "covered",
          "evidence": "tests/passthrough.rs#messages_forwards_anthropic_headers_verbatim_and_preserves_query"
        },
        "stream": {
          "status": "covered",
          "evidence": "tests/passthrough.rs#relays_sse_response_with_content_type_preserved"
        },
        "tools": {
          "status": "covered",
          "evidence": "tests/inbound_anthropic_translation.rs#enabled_collaboration_translates_and_restores_json_tool_call"
        },
        "terminal": {
          "status": "covered",
          "evidence": "tests/inbound_anthropic_translation.rs#response_premature_stream_eof_is_failed_not_completed"
        },
        "malformed": {
          "status": "covered",
          "evidence": "tests/passthrough.rs#messages_rejects_duplicate_top_level_model_fields"
        },
        "truncation": {
          "status": "covered",
          "evidence": "tests/failover.rs#redispatch_gate_client_visible_truncated_body_stops_without_replaying_turn"
        },
        "auth": {
          "status": "covered",
          "evidence": "tests/passthrough.rs#messages_forwards_incoming_credentials_unchanged"
        },
        "cancellation": {
          "status": "covered",
          "evidence": "tests/passthrough.rs#response_drop_releases_upstream_work_and_global_capacity"
        },
        "retry": {
          "status": "covered",
          "evidence": "tests/retry.rs#messages_path_does_not_retry_transient_status_despite_enabled_policy"
        }
      },
      "provenance": {
        "class": "source",
        "repository": "shunt",
        "revision": "4b7b5bae1956a304b0ac5359d8dd61124c6728dc",
        "inspected": "2026-09-09",
        "sanitization": "sanitized: no credentials, account identifiers, or private content"
      },
      "capture": "none",
      "live": "none",
      "evidence_scope": "Named tests exercise the shared adapter/auth contract and synthetic fixtures, not live service availability or entitlement for this model. Cancellation includes terminal-failure or process-shutdown cleanup where named; client-disconnect coverage is separately identifiable from the cited test."
    },
    {
      "provider": "anthropic",
      "auth": "passthrough",
      "model": "claude-opus-4-7",
      "model_policy": "Exact finite builtin discovery catalog (src/discovery.rs BUILTIN_MODELS); other client-supplied ids pass through without a support claim.",
      "wire": "https://api.anthropic.com/v1/messages",
      "scenarios": {
        "normal": {
          "status": "covered",
          "evidence": "tests/passthrough.rs#messages_forwards_anthropic_headers_verbatim_and_preserves_query"
        },
        "stream": {
          "status": "covered",
          "evidence": "tests/passthrough.rs#relays_sse_response_with_content_type_preserved"
        },
        "tools": {
          "status": "covered",
          "evidence": "tests/inbound_anthropic_translation.rs#enabled_collaboration_translates_and_restores_json_tool_call"
        },
        "terminal": {
          "status": "covered",
          "evidence": "tests/inbound_anthropic_translation.rs#response_premature_stream_eof_is_failed_not_completed"
        },
        "malformed": {
          "status": "covered",
          "evidence": "tests/passthrough.rs#messages_rejects_duplicate_top_level_model_fields"
        },
        "truncation": {
          "status": "covered",
          "evidence": "tests/failover.rs#redispatch_gate_client_visible_truncated_body_stops_without_replaying_turn"
        },
        "auth": {
          "status": "covered",
          "evidence": "tests/passthrough.rs#messages_forwards_incoming_credentials_unchanged"
        },
        "cancellation": {
          "status": "covered",
          "evidence": "tests/passthrough.rs#response_drop_releases_upstream_work_and_global_capacity"
        },
        "retry": {
          "status": "covered",
          "evidence": "tests/retry.rs#messages_path_does_not_retry_transient_status_despite_enabled_policy"
        }
      },
      "provenance": {
        "class": "source",
        "repository": "shunt",
        "revision": "4b7b5bae1956a304b0ac5359d8dd61124c6728dc",
        "inspected": "2026-09-09",
        "sanitization": "sanitized: no credentials, account identifiers, or private content"
      },
      "capture": "none",
      "live": "none",
      "evidence_scope": "Named tests exercise the shared adapter/auth contract and synthetic fixtures, not live service availability or entitlement for this model. Cancellation includes terminal-failure or process-shutdown cleanup where named; client-disconnect coverage is separately identifiable from the cited test."
    },
    {
      "provider": "anthropic",
      "auth": "passthrough",
      "model": "claude-opus-4-8",
      "model_policy": "Exact finite builtin discovery catalog (src/discovery.rs BUILTIN_MODELS); other client-supplied ids pass through without a support claim.",
      "wire": "https://api.anthropic.com/v1/messages",
      "scenarios": {
        "normal": {
          "status": "covered",
          "evidence": "tests/passthrough.rs#messages_forwards_anthropic_headers_verbatim_and_preserves_query"
        },
        "stream": {
          "status": "covered",
          "evidence": "tests/passthrough.rs#relays_sse_response_with_content_type_preserved"
        },
        "tools": {
          "status": "covered",
          "evidence": "tests/inbound_anthropic_translation.rs#enabled_collaboration_translates_and_restores_json_tool_call"
        },
        "terminal": {
          "status": "covered",
          "evidence": "tests/inbound_anthropic_translation.rs#response_premature_stream_eof_is_failed_not_completed"
        },
        "malformed": {
          "status": "covered",
          "evidence": "tests/passthrough.rs#messages_rejects_duplicate_top_level_model_fields"
        },
        "truncation": {
          "status": "covered",
          "evidence": "tests/failover.rs#redispatch_gate_client_visible_truncated_body_stops_without_replaying_turn"
        },
        "auth": {
          "status": "covered",
          "evidence": "tests/passthrough.rs#messages_forwards_incoming_credentials_unchanged"
        },
        "cancellation": {
          "status": "covered",
          "evidence": "tests/passthrough.rs#response_drop_releases_upstream_work_and_global_capacity"
        },
        "retry": {
          "status": "covered",
          "evidence": "tests/retry.rs#messages_path_does_not_retry_transient_status_despite_enabled_policy"
        }
      },
      "provenance": {
        "class": "source",
        "repository": "shunt",
        "revision": "4b7b5bae1956a304b0ac5359d8dd61124c6728dc",
        "inspected": "2026-09-09",
        "sanitization": "sanitized: no credentials, account identifiers, or private content"
      },
      "capture": "none",
      "live": "none",
      "evidence_scope": "Named tests exercise the shared adapter/auth contract and synthetic fixtures, not live service availability or entitlement for this model. Cancellation includes terminal-failure or process-shutdown cleanup where named; client-disconnect coverage is separately identifiable from the cited test."
    },
    {
      "provider": "anthropic",
      "auth": "passthrough",
      "model": "claude-opus-5",
      "model_policy": "Exact finite builtin discovery catalog (src/discovery.rs BUILTIN_MODELS); other client-supplied ids pass through without a support claim.",
      "wire": "https://api.anthropic.com/v1/messages",
      "scenarios": {
        "normal": {
          "status": "covered",
          "evidence": "tests/passthrough.rs#messages_forwards_anthropic_headers_verbatim_and_preserves_query"
        },
        "stream": {
          "status": "covered",
          "evidence": "tests/passthrough.rs#relays_sse_response_with_content_type_preserved"
        },
        "tools": {
          "status": "covered",
          "evidence": "tests/inbound_anthropic_translation.rs#enabled_collaboration_translates_and_restores_json_tool_call"
        },
        "terminal": {
          "status": "covered",
          "evidence": "tests/inbound_anthropic_translation.rs#response_premature_stream_eof_is_failed_not_completed"
        },
        "malformed": {
          "status": "covered",
          "evidence": "tests/passthrough.rs#messages_rejects_duplicate_top_level_model_fields"
        },
        "truncation": {
          "status": "covered",
          "evidence": "tests/failover.rs#redispatch_gate_client_visible_truncated_body_stops_without_replaying_turn"
        },
        "auth": {
          "status": "covered",
          "evidence": "tests/passthrough.rs#messages_forwards_incoming_credentials_unchanged"
        },
        "cancellation": {
          "status": "covered",
          "evidence": "tests/passthrough.rs#response_drop_releases_upstream_work_and_global_capacity"
        },
        "retry": {
          "status": "covered",
          "evidence": "tests/retry.rs#messages_path_does_not_retry_transient_status_despite_enabled_policy"
        }
      },
      "provenance": {
        "class": "source",
        "repository": "shunt",
        "revision": "4b7b5bae1956a304b0ac5359d8dd61124c6728dc",
        "inspected": "2026-09-09",
        "sanitization": "sanitized: no credentials, account identifiers, or private content"
      },
      "capture": "none",
      "live": "none",
      "evidence_scope": "Named tests exercise the shared adapter/auth contract and synthetic fixtures, not live service availability or entitlement for this model. Cancellation includes terminal-failure or process-shutdown cleanup where named; client-disconnect coverage is separately identifiable from the cited test."
    },
    {
      "provider": "anthropic",
      "auth": "passthrough",
      "model": "claude-sonnet-4-5-20250929",
      "model_policy": "Exact finite builtin discovery catalog (src/discovery.rs BUILTIN_MODELS); other client-supplied ids pass through without a support claim.",
      "wire": "https://api.anthropic.com/v1/messages",
      "scenarios": {
        "normal": {
          "status": "covered",
          "evidence": "tests/passthrough.rs#messages_forwards_anthropic_headers_verbatim_and_preserves_query"
        },
        "stream": {
          "status": "covered",
          "evidence": "tests/passthrough.rs#relays_sse_response_with_content_type_preserved"
        },
        "tools": {
          "status": "covered",
          "evidence": "tests/inbound_anthropic_translation.rs#enabled_collaboration_translates_and_restores_json_tool_call"
        },
        "terminal": {
          "status": "covered",
          "evidence": "tests/inbound_anthropic_translation.rs#response_premature_stream_eof_is_failed_not_completed"
        },
        "malformed": {
          "status": "covered",
          "evidence": "tests/passthrough.rs#messages_rejects_duplicate_top_level_model_fields"
        },
        "truncation": {
          "status": "covered",
          "evidence": "tests/failover.rs#redispatch_gate_client_visible_truncated_body_stops_without_replaying_turn"
        },
        "auth": {
          "status": "covered",
          "evidence": "tests/passthrough.rs#messages_forwards_incoming_credentials_unchanged"
        },
        "cancellation": {
          "status": "covered",
          "evidence": "tests/passthrough.rs#response_drop_releases_upstream_work_and_global_capacity"
        },
        "retry": {
          "status": "covered",
          "evidence": "tests/retry.rs#messages_path_does_not_retry_transient_status_despite_enabled_policy"
        }
      },
      "provenance": {
        "class": "source",
        "repository": "shunt",
        "revision": "4b7b5bae1956a304b0ac5359d8dd61124c6728dc",
        "inspected": "2026-09-09",
        "sanitization": "sanitized: no credentials, account identifiers, or private content"
      },
      "capture": "none",
      "live": "none",
      "evidence_scope": "Named tests exercise the shared adapter/auth contract and synthetic fixtures, not live service availability or entitlement for this model. Cancellation includes terminal-failure or process-shutdown cleanup where named; client-disconnect coverage is separately identifiable from the cited test."
    },
    {
      "provider": "anthropic",
      "auth": "passthrough",
      "model": "claude-sonnet-4-6",
      "model_policy": "Exact finite builtin discovery catalog (src/discovery.rs BUILTIN_MODELS); other client-supplied ids pass through without a support claim.",
      "wire": "https://api.anthropic.com/v1/messages",
      "scenarios": {
        "normal": {
          "status": "covered",
          "evidence": "tests/passthrough.rs#messages_forwards_anthropic_headers_verbatim_and_preserves_query"
        },
        "stream": {
          "status": "covered",
          "evidence": "tests/passthrough.rs#relays_sse_response_with_content_type_preserved"
        },
        "tools": {
          "status": "covered",
          "evidence": "tests/inbound_anthropic_translation.rs#enabled_collaboration_translates_and_restores_json_tool_call"
        },
        "terminal": {
          "status": "covered",
          "evidence": "tests/inbound_anthropic_translation.rs#response_premature_stream_eof_is_failed_not_completed"
        },
        "malformed": {
          "status": "covered",
          "evidence": "tests/passthrough.rs#messages_rejects_duplicate_top_level_model_fields"
        },
        "truncation": {
          "status": "covered",
          "evidence": "tests/failover.rs#redispatch_gate_client_visible_truncated_body_stops_without_replaying_turn"
        },
        "auth": {
          "status": "covered",
          "evidence": "tests/passthrough.rs#messages_forwards_incoming_credentials_unchanged"
        },
        "cancellation": {
          "status": "covered",
          "evidence": "tests/passthrough.rs#response_drop_releases_upstream_work_and_global_capacity"
        },
        "retry": {
          "status": "covered",
          "evidence": "tests/retry.rs#messages_path_does_not_retry_transient_status_despite_enabled_policy"
        }
      },
      "provenance": {
        "class": "source",
        "repository": "shunt",
        "revision": "4b7b5bae1956a304b0ac5359d8dd61124c6728dc",
        "inspected": "2026-09-09",
        "sanitization": "sanitized: no credentials, account identifiers, or private content"
      },
      "capture": "none",
      "live": "none",
      "evidence_scope": "Named tests exercise the shared adapter/auth contract and synthetic fixtures, not live service availability or entitlement for this model. Cancellation includes terminal-failure or process-shutdown cleanup where named; client-disconnect coverage is separately identifiable from the cited test."
    },
    {
      "provider": "anthropic",
      "auth": "passthrough",
      "model": "claude-sonnet-5",
      "model_policy": "Exact finite builtin discovery catalog (src/discovery.rs BUILTIN_MODELS); other client-supplied ids pass through without a support claim.",
      "wire": "https://api.anthropic.com/v1/messages",
      "scenarios": {
        "normal": {
          "status": "covered",
          "evidence": "tests/passthrough.rs#messages_forwards_anthropic_headers_verbatim_and_preserves_query"
        },
        "stream": {
          "status": "covered",
          "evidence": "tests/passthrough.rs#relays_sse_response_with_content_type_preserved"
        },
        "tools": {
          "status": "covered",
          "evidence": "tests/inbound_anthropic_translation.rs#enabled_collaboration_translates_and_restores_json_tool_call"
        },
        "terminal": {
          "status": "covered",
          "evidence": "tests/inbound_anthropic_translation.rs#response_premature_stream_eof_is_failed_not_completed"
        },
        "malformed": {
          "status": "covered",
          "evidence": "tests/passthrough.rs#messages_rejects_duplicate_top_level_model_fields"
        },
        "truncation": {
          "status": "covered",
          "evidence": "tests/failover.rs#redispatch_gate_client_visible_truncated_body_stops_without_replaying_turn"
        },
        "auth": {
          "status": "covered",
          "evidence": "tests/passthrough.rs#messages_forwards_incoming_credentials_unchanged"
        },
        "cancellation": {
          "status": "covered",
          "evidence": "tests/passthrough.rs#response_drop_releases_upstream_work_and_global_capacity"
        },
        "retry": {
          "status": "covered",
          "evidence": "tests/retry.rs#messages_path_does_not_retry_transient_status_despite_enabled_policy"
        }
      },
      "provenance": {
        "class": "source",
        "repository": "shunt",
        "revision": "4b7b5bae1956a304b0ac5359d8dd61124c6728dc",
        "inspected": "2026-09-09",
        "sanitization": "sanitized: no credentials, account identifiers, or private content"
      },
      "capture": "none",
      "live": "none",
      "evidence_scope": "Named tests exercise the shared adapter/auth contract and synthetic fixtures, not live service availability or entitlement for this model. Cancellation includes terminal-failure or process-shutdown cleanup where named; client-disconnect coverage is separately identifiable from the cited test."
    },
    {
      "provider": "antigravity",
      "auth": "antigravity_oauth",
      "model": "gemini-3.8-flash-high",
      "model_policy": "Exact model/effort admission comes from fetchAvailableModels catalog evidence only; no static model guessing.",
      "wire": "https://daily-cloudcode-pa.googleapis.com/v1internal:streamGenerateContent?alt=sse",
      "scenarios": {
        "normal": {
          "status": "covered",
          "evidence": "tests/gemini_conformance.rs#antigravity_native_sse_real_loopback_both_downstream_modes"
        },
        "stream": {
          "status": "covered",
          "evidence": "tests/gemini_conformance.rs#antigravity_native_sse_real_loopback_both_downstream_modes"
        },
        "tools": {
          "status": "covered",
          "evidence": "tests/gemini_conformance.rs#gemini_no_post_header_replay_preserves_tool_call_result_pairing_next_turn"
        },
        "terminal": {
          "status": "covered",
          "evidence": "tests/gemini_conformance.rs#antigravity_native_sse_strict_failures_are_closed_in_both_modes"
        },
        "malformed": {
          "status": "covered",
          "evidence": "tests/gemini_conformance.rs#antigravity_native_sse_strict_failures_are_closed_in_both_modes"
        },
        "truncation": {
          "status": "covered",
          "evidence": "tests/gemini_conformance.rs#antigravity_native_sse_strict_failures_are_closed_in_both_modes"
        },
        "auth": {
          "status": "covered",
          "evidence": "tests/check_cli.rs#check_refuses_a_routed_antigravity_provider_without_a_credential"
        },
        "cancellation": {
          "status": "covered",
          "evidence": "tests/gemini_conformance.rs#antigravity_native_lifetime_unary_cancellation_releases_upstream_and_capacity"
        },
        "retry": {
          "status": "covered",
          "evidence": "tests/gemini_conformance.rs#gemini_identity_retry_never_retries_returned_transient_statuses"
        }
      },
      "provenance": {
        "class": "source",
        "repository": "shunt",
        "revision": "4b7b5bae1956a304b0ac5359d8dd61124c6728dc",
        "inspected": "2026-09-09",
        "sanitization": "sanitized: no credentials, account identifiers, or private content"
      },
      "capture": "none",
      "live": "none",
      "evidence_scope": "Named tests exercise the shared adapter/auth contract and synthetic fixtures, not live service availability or entitlement for this model. Cancellation includes terminal-failure or process-shutdown cleanup where named; client-disconnect coverage is separately identifiable from the cited test."
    },
    {
      "provider": "codex",
      "auth": "chatgpt_oauth",
      "model": "gpt-5.2",
      "wire": "https://chatgpt.com/backend-api/codex/responses",
      "scenarios": {
        "normal": {
          "status": "covered",
          "evidence": "tests/inbound_codex_endpoint.rs#exact_native_route"
        },
        "stream": {
          "status": "covered",
          "evidence": "tests/inbound_codex_endpoint.rs#sse_response_is_relayed_verbatim_without_translation"
        },
        "tools": {
          "status": "covered",
          "evidence": "tests/responses_translate.rs#preserves_tool_use_and_tool_result_call_ids"
        },
        "terminal": {
          "status": "covered",
          "evidence": "tests/inbound_codex_endpoint.rs#responses_terminal_gateway_owned_401_body_is_openai_shaped"
        },
        "malformed": {
          "status": "covered",
          "evidence": "tests/inbound_codex_endpoint.rs#body_limit_error_uses_openai_responses_shape"
        },
        "truncation": {
          "status": "covered",
          "evidence": "src/adapters/responses/http.rs#responses_transport_terminal_http_rejects_a_truncated_upstream"
        },
        "auth": {
          "status": "covered",
          "evidence": "tests/inbound_codex_endpoint.rs#native_provider_auth"
        },
        "cancellation": {
          "status": "covered",
          "evidence": "tests/codex_multi_account.rs#response_drop_releases_account_admission_and_cancels_upstream_work"
        },
        "retry": {
          "status": "covered",
          "evidence": "tests/inbound_codex_endpoint.rs#refresh_retry_refreshes_then_relays_verbatim"
        }
      },
      "provenance": {
        "class": "source",
        "repository": "shunt",
        "revision": "4b7b5bae1956a304b0ac5359d8dd61124c6728dc",
        "inspected": "2026-09-09",
        "sanitization": "sanitized: no credentials, account identifiers, or private content"
      },
      "capture": "none",
      "live": "none",
      "evidence_scope": "Named tests exercise the shared adapter/auth contract and synthetic fixtures, not live service availability or entitlement for this model. Cancellation includes terminal-failure or process-shutdown cleanup where named; client-disconnect coverage is separately identifiable from the cited test."
    },
    {
      "provider": "codex",
      "auth": "chatgpt_oauth",
      "model": "gpt-5.4",
      "wire": "https://chatgpt.com/backend-api/codex/responses",
      "scenarios": {
        "normal": {
          "status": "covered",
          "evidence": "tests/inbound_codex_endpoint.rs#exact_native_route"
        },
        "stream": {
          "status": "covered",
          "evidence": "tests/inbound_codex_endpoint.rs#sse_response_is_relayed_verbatim_without_translation"
        },
        "tools": {
          "status": "covered",
          "evidence": "tests/responses_translate.rs#preserves_tool_use_and_tool_result_call_ids"
        },
        "terminal": {
          "status": "covered",
          "evidence": "tests/inbound_codex_endpoint.rs#responses_terminal_gateway_owned_401_body_is_openai_shaped"
        },
        "malformed": {
          "status": "covered",
          "evidence": "tests/inbound_codex_endpoint.rs#body_limit_error_uses_openai_responses_shape"
        },
        "truncation": {
          "status": "covered",
          "evidence": "src/adapters/responses/http.rs#responses_transport_terminal_http_rejects_a_truncated_upstream"
        },
        "auth": {
          "status": "covered",
          "evidence": "tests/inbound_codex_endpoint.rs#native_provider_auth"
        },
        "cancellation": {
          "status": "covered",
          "evidence": "tests/codex_multi_account.rs#response_drop_releases_account_admission_and_cancels_upstream_work"
        },
        "retry": {
          "status": "covered",
          "evidence": "tests/inbound_codex_endpoint.rs#refresh_retry_refreshes_then_relays_verbatim"
        }
      },
      "provenance": {
        "class": "source",
        "repository": "shunt",
        "revision": "4b7b5bae1956a304b0ac5359d8dd61124c6728dc",
        "inspected": "2026-09-09",
        "sanitization": "sanitized: no credentials, account identifiers, or private content"
      },
      "capture": "none",
      "live": "none",
      "evidence_scope": "Named tests exercise the shared adapter/auth contract and synthetic fixtures, not live service availability or entitlement for this model. Cancellation includes terminal-failure or process-shutdown cleanup where named; client-disconnect coverage is separately identifiable from the cited test."
    },
    {
      "provider": "codex",
      "auth": "chatgpt_oauth",
      "model": "gpt-5.4-mini",
      "wire": "https://chatgpt.com/backend-api/codex/responses",
      "scenarios": {
        "normal": {
          "status": "covered",
          "evidence": "tests/inbound_codex_endpoint.rs#exact_native_route"
        },
        "stream": {
          "status": "covered",
          "evidence": "tests/inbound_codex_endpoint.rs#sse_response_is_relayed_verbatim_without_translation"
        },
        "tools": {
          "status": "covered",
          "evidence": "tests/responses_translate.rs#preserves_tool_use_and_tool_result_call_ids"
        },
        "terminal": {
          "status": "covered",
          "evidence": "tests/inbound_codex_endpoint.rs#responses_terminal_gateway_owned_401_body_is_openai_shaped"
        },
        "malformed": {
          "status": "covered",
          "evidence": "tests/inbound_codex_endpoint.rs#body_limit_error_uses_openai_responses_shape"
        },
        "truncation": {
          "status": "covered",
          "evidence": "src/adapters/responses/http.rs#responses_transport_terminal_http_rejects_a_truncated_upstream"
        },
        "auth": {
          "status": "covered",
          "evidence": "tests/inbound_codex_endpoint.rs#native_provider_auth"
        },
        "cancellation": {
          "status": "covered",
          "evidence": "tests/codex_multi_account.rs#response_drop_releases_account_admission_and_cancels_upstream_work"
        },
        "retry": {
          "status": "covered",
          "evidence": "tests/inbound_codex_endpoint.rs#refresh_retry_refreshes_then_relays_verbatim"
        }
      },
      "provenance": {
        "class": "source",
        "repository": "shunt",
        "revision": "4b7b5bae1956a304b0ac5359d8dd61124c6728dc",
        "inspected": "2026-09-09",
        "sanitization": "sanitized: no credentials, account identifiers, or private content"
      },
      "capture": "none",
      "live": "none",
      "evidence_scope": "Named tests exercise the shared adapter/auth contract and synthetic fixtures, not live service availability or entitlement for this model. Cancellation includes terminal-failure or process-shutdown cleanup where named; client-disconnect coverage is separately identifiable from the cited test."
    },
    {
      "provider": "codex",
      "auth": "chatgpt_oauth",
      "model": "gpt-5.5",
      "wire": "https://chatgpt.com/backend-api/codex/responses",
      "scenarios": {
        "normal": {
          "status": "covered",
          "evidence": "tests/inbound_codex_endpoint.rs#exact_native_route"
        },
        "stream": {
          "status": "covered",
          "evidence": "tests/inbound_codex_endpoint.rs#sse_response_is_relayed_verbatim_without_translation"
        },
        "tools": {
          "status": "covered",
          "evidence": "tests/responses_translate.rs#preserves_tool_use_and_tool_result_call_ids"
        },
        "terminal": {
          "status": "covered",
          "evidence": "tests/inbound_codex_endpoint.rs#responses_terminal_gateway_owned_401_body_is_openai_shaped"
        },
        "malformed": {
          "status": "covered",
          "evidence": "tests/inbound_codex_endpoint.rs#body_limit_error_uses_openai_responses_shape"
        },
        "truncation": {
          "status": "covered",
          "evidence": "src/adapters/responses/http.rs#responses_transport_terminal_http_rejects_a_truncated_upstream"
        },
        "auth": {
          "status": "covered",
          "evidence": "tests/inbound_codex_endpoint.rs#native_provider_auth"
        },
        "cancellation": {
          "status": "covered",
          "evidence": "tests/codex_multi_account.rs#response_drop_releases_account_admission_and_cancels_upstream_work"
        },
        "retry": {
          "status": "covered",
          "evidence": "tests/inbound_codex_endpoint.rs#refresh_retry_refreshes_then_relays_verbatim"
        }
      },
      "provenance": {
        "class": "source",
        "repository": "shunt",
        "revision": "4b7b5bae1956a304b0ac5359d8dd61124c6728dc",
        "inspected": "2026-09-09",
        "sanitization": "sanitized: no credentials, account identifiers, or private content"
      },
      "capture": "none",
      "live": "none",
      "evidence_scope": "Named tests exercise the shared adapter/auth contract and synthetic fixtures, not live service availability or entitlement for this model. Cancellation includes terminal-failure or process-shutdown cleanup where named; client-disconnect coverage is separately identifiable from the cited test."
    },
    {
      "provider": "codex",
      "auth": "chatgpt_oauth",
      "model": "gpt-5.6-luna",
      "wire": "https://chatgpt.com/backend-api/codex/responses",
      "scenarios": {
        "normal": {
          "status": "covered",
          "evidence": "tests/inbound_codex_endpoint.rs#exact_native_route"
        },
        "stream": {
          "status": "covered",
          "evidence": "tests/inbound_codex_endpoint.rs#sse_response_is_relayed_verbatim_without_translation"
        },
        "tools": {
          "status": "covered",
          "evidence": "tests/responses_translate.rs#preserves_tool_use_and_tool_result_call_ids"
        },
        "terminal": {
          "status": "covered",
          "evidence": "tests/inbound_codex_endpoint.rs#responses_terminal_gateway_owned_401_body_is_openai_shaped"
        },
        "malformed": {
          "status": "covered",
          "evidence": "tests/inbound_codex_endpoint.rs#body_limit_error_uses_openai_responses_shape"
        },
        "truncation": {
          "status": "covered",
          "evidence": "src/adapters/responses/http.rs#responses_transport_terminal_http_rejects_a_truncated_upstream"
        },
        "auth": {
          "status": "covered",
          "evidence": "tests/inbound_codex_endpoint.rs#native_provider_auth"
        },
        "cancellation": {
          "status": "covered",
          "evidence": "tests/codex_multi_account.rs#response_drop_releases_account_admission_and_cancels_upstream_work"
        },
        "retry": {
          "status": "covered",
          "evidence": "tests/inbound_codex_endpoint.rs#refresh_retry_refreshes_then_relays_verbatim"
        }
      },
      "provenance": {
        "class": "source",
        "repository": "shunt",
        "revision": "4b7b5bae1956a304b0ac5359d8dd61124c6728dc",
        "inspected": "2026-09-09",
        "sanitization": "sanitized: no credentials, account identifiers, or private content"
      },
      "capture": "none",
      "live": "none",
      "evidence_scope": "Named tests exercise the shared adapter/auth contract and synthetic fixtures, not live service availability or entitlement for this model. Cancellation includes terminal-failure or process-shutdown cleanup where named; client-disconnect coverage is separately identifiable from the cited test."
    },
    {
      "provider": "codex",
      "auth": "chatgpt_oauth",
      "model": "gpt-5.6-sol",
      "wire": "https://chatgpt.com/backend-api/codex/responses",
      "scenarios": {
        "normal": {
          "status": "covered",
          "evidence": "tests/inbound_codex_endpoint.rs#exact_native_route"
        },
        "stream": {
          "status": "covered",
          "evidence": "tests/inbound_codex_endpoint.rs#sse_response_is_relayed_verbatim_without_translation"
        },
        "tools": {
          "status": "covered",
          "evidence": "tests/responses_translate.rs#preserves_tool_use_and_tool_result_call_ids"
        },
        "terminal": {
          "status": "covered",
          "evidence": "tests/inbound_codex_endpoint.rs#responses_terminal_gateway_owned_401_body_is_openai_shaped"
        },
        "malformed": {
          "status": "covered",
          "evidence": "tests/inbound_codex_endpoint.rs#body_limit_error_uses_openai_responses_shape"
        },
        "truncation": {
          "status": "covered",
          "evidence": "src/adapters/responses/http.rs#responses_transport_terminal_http_rejects_a_truncated_upstream"
        },
        "auth": {
          "status": "covered",
          "evidence": "tests/inbound_codex_endpoint.rs#native_provider_auth"
        },
        "cancellation": {
          "status": "covered",
          "evidence": "tests/codex_multi_account.rs#response_drop_releases_account_admission_and_cancels_upstream_work"
        },
        "retry": {
          "status": "covered",
          "evidence": "tests/inbound_codex_endpoint.rs#refresh_retry_refreshes_then_relays_verbatim"
        }
      },
      "provenance": {
        "class": "source",
        "repository": "shunt",
        "revision": "4b7b5bae1956a304b0ac5359d8dd61124c6728dc",
        "inspected": "2026-09-09",
        "sanitization": "sanitized: no credentials, account identifiers, or private content"
      },
      "capture": "none",
      "live": "none",
      "evidence_scope": "Named tests exercise the shared adapter/auth contract and synthetic fixtures, not live service availability or entitlement for this model. Cancellation includes terminal-failure or process-shutdown cleanup where named; client-disconnect coverage is separately identifiable from the cited test."
    },
    {
      "provider": "codex",
      "auth": "chatgpt_oauth",
      "model": "gpt-5.6-terra",
      "wire": "https://chatgpt.com/backend-api/codex/responses",
      "scenarios": {
        "normal": {
          "status": "covered",
          "evidence": "tests/inbound_codex_endpoint.rs#exact_native_route"
        },
        "stream": {
          "status": "covered",
          "evidence": "tests/inbound_codex_endpoint.rs#sse_response_is_relayed_verbatim_without_translation"
        },
        "tools": {
          "status": "covered",
          "evidence": "tests/responses_translate.rs#preserves_tool_use_and_tool_result_call_ids"
        },
        "terminal": {
          "status": "covered",
          "evidence": "tests/inbound_codex_endpoint.rs#responses_terminal_gateway_owned_401_body_is_openai_shaped"
        },
        "malformed": {
          "status": "covered",
          "evidence": "tests/inbound_codex_endpoint.rs#body_limit_error_uses_openai_responses_shape"
        },
        "truncation": {
          "status": "covered",
          "evidence": "src/adapters/responses/http.rs#responses_transport_terminal_http_rejects_a_truncated_upstream"
        },
        "auth": {
          "status": "covered",
          "evidence": "tests/inbound_codex_endpoint.rs#native_provider_auth"
        },
        "cancellation": {
          "status": "covered",
          "evidence": "tests/codex_multi_account.rs#response_drop_releases_account_admission_and_cancels_upstream_work"
        },
        "retry": {
          "status": "covered",
          "evidence": "tests/inbound_codex_endpoint.rs#refresh_retry_refreshes_then_relays_verbatim"
        }
      },
      "provenance": {
        "class": "source",
        "repository": "shunt",
        "revision": "4b7b5bae1956a304b0ac5359d8dd61124c6728dc",
        "inspected": "2026-09-09",
        "sanitization": "sanitized: no credentials, account identifiers, or private content"
      },
      "capture": "none",
      "live": "none",
      "evidence_scope": "Named tests exercise the shared adapter/auth contract and synthetic fixtures, not live service availability or entitlement for this model. Cancellation includes terminal-failure or process-shutdown cleanup where named; client-disconnect coverage is separately identifiable from the cited test."
    },
    {
      "provider": "codex",
      "auth": "chatgpt_oauth",
      "model": "gpt-6-astra",
      "wire": "https://chatgpt.com/backend-api/codex/responses",
      "scenarios": {
        "normal": {
          "status": "covered",
          "evidence": "tests/inbound_codex_endpoint.rs#exact_native_route"
        },
        "stream": {
          "status": "covered",
          "evidence": "tests/inbound_codex_endpoint.rs#sse_response_is_relayed_verbatim_without_translation"
        },
        "tools": {
          "status": "covered",
          "evidence": "tests/responses_translate.rs#preserves_tool_use_and_tool_result_call_ids"
        },
        "terminal": {
          "status": "covered",
          "evidence": "tests/inbound_codex_endpoint.rs#responses_terminal_gateway_owned_401_body_is_openai_shaped"
        },
        "malformed": {
          "status": "covered",
          "evidence": "tests/inbound_codex_endpoint.rs#body_limit_error_uses_openai_responses_shape"
        },
        "truncation": {
          "status": "covered",
          "evidence": "src/adapters/responses/http.rs#responses_transport_terminal_http_rejects_a_truncated_upstream"
        },
        "auth": {
          "status": "covered",
          "evidence": "tests/inbound_codex_endpoint.rs#native_provider_auth"
        },
        "cancellation": {
          "status": "covered",
          "evidence": "tests/codex_multi_account.rs#response_drop_releases_account_admission_and_cancels_upstream_work"
        },
        "retry": {
          "status": "covered",
          "evidence": "tests/inbound_codex_endpoint.rs#refresh_retry_refreshes_then_relays_verbatim"
        }
      },
      "provenance": {
        "class": "source",
        "repository": "shunt",
        "revision": "4b7b5bae1956a304b0ac5359d8dd61124c6728dc",
        "inspected": "2026-09-09",
        "sanitization": "sanitized: no credentials, account identifiers, or private content"
      },
      "capture": "none",
      "live": "none",
      "evidence_scope": "Named tests exercise the shared adapter/auth contract and synthetic fixtures, not live service availability or entitlement for this model. Cancellation includes terminal-failure or process-shutdown cleanup where named; client-disconnect coverage is separately identifiable from the cited test."
    },
    {
      "provider": "command-code",
      "auth": "command_code_oauth",
      "model": "deepseek/deepseek-v4-flash",
      "wire": "https://api.commandcode.ai/alpha/generate",
      "scenarios": {
        "normal": {
          "status": "covered",
          "evidence": "src/adapters/command_code/router_tests.rs#command_code_tracer_unary"
        },
        "stream": {
          "status": "covered",
          "evidence": "src/adapters/command_code/router_tests.rs#command_code_tracer_response_incremental_and_unary"
        },
        "tools": {
          "status": "covered",
          "evidence": "tests/command_code_translate.rs#command_code_translate_tools_images_orphans_and_ordering"
        },
        "terminal": {
          "status": "covered",
          "evidence": "src/adapters/command_code/router_tests.rs#command_code_tracer_terminal_grammar"
        },
        "malformed": {
          "status": "covered",
          "evidence": "tests/command_code_translate.rs#command_code_translate_envelope_defaults_and_rejections"
        },
        "truncation": {
          "status": "covered",
          "evidence": "src/adapters/command_code/router_tests.rs#command_code_tracer_response_errors_never_emit_success"
        },
        "auth": {
          "status": "covered",
          "evidence": "src/auth/command_code/tests/matrix.rs#command_code_matrix_auth_shapes_file_invariance_and_byte_bounds"
        },
        "cancellation": {
          "status": "covered",
          "evidence": "src/adapters/command_code/router_tests/lifetime.rs#command_code_lifetime_cancel_mid_body_streaming"
        },
        "retry": {
          "status": "covered",
          "evidence": "src/adapters/command_code/router_tests/replay.rs#command_code_lifetime_replay_post_send_timeout_once"
        }
      },
      "model_evidence": "tests/command_code_translate.rs#command_code_translate_effort_exact_table",
      "provenance": {
        "class": "source",
        "repository": "opencodex",
        "revision": "055c3ecf0de6c35f59195fc434d6b08525182b7f",
        "inspected": "2026-09-08",
        "sanitization": "sanitized: exact model/effort tuples only; translated material noticed in THIRD-PARTY-NOTICES.md"
      },
      "capture": "none",
      "live": "none",
      "evidence_scope": "Named tests exercise the shared adapter/auth contract and synthetic fixtures, not live service availability or entitlement for this model. Cancellation includes terminal-failure or process-shutdown cleanup where named; client-disconnect coverage is separately identifiable from the cited test."
    },
    {
      "provider": "command-code",
      "auth": "command_code_oauth",
      "model": "deepseek/deepseek-v4-pro",
      "wire": "https://api.commandcode.ai/alpha/generate",
      "scenarios": {
        "normal": {
          "status": "covered",
          "evidence": "src/adapters/command_code/router_tests.rs#command_code_tracer_unary"
        },
        "stream": {
          "status": "covered",
          "evidence": "src/adapters/command_code/router_tests.rs#command_code_tracer_response_incremental_and_unary"
        },
        "tools": {
          "status": "covered",
          "evidence": "tests/command_code_translate.rs#command_code_translate_tools_images_orphans_and_ordering"
        },
        "terminal": {
          "status": "covered",
          "evidence": "src/adapters/command_code/router_tests.rs#command_code_tracer_terminal_grammar"
        },
        "malformed": {
          "status": "covered",
          "evidence": "tests/command_code_translate.rs#command_code_translate_envelope_defaults_and_rejections"
        },
        "truncation": {
          "status": "covered",
          "evidence": "src/adapters/command_code/router_tests.rs#command_code_tracer_response_errors_never_emit_success"
        },
        "auth": {
          "status": "covered",
          "evidence": "src/auth/command_code/tests/matrix.rs#command_code_matrix_auth_shapes_file_invariance_and_byte_bounds"
        },
        "cancellation": {
          "status": "covered",
          "evidence": "src/adapters/command_code/router_tests/lifetime.rs#command_code_lifetime_cancel_mid_body_streaming"
        },
        "retry": {
          "status": "covered",
          "evidence": "src/adapters/command_code/router_tests/replay.rs#command_code_lifetime_replay_post_send_timeout_once"
        }
      },
      "model_evidence": "tests/command_code_translate.rs#command_code_translate_effort_exact_table",
      "provenance": {
        "class": "source",
        "repository": "opencodex",
        "revision": "055c3ecf0de6c35f59195fc434d6b08525182b7f",
        "inspected": "2026-09-08",
        "sanitization": "sanitized: exact model/effort tuples only; translated material noticed in THIRD-PARTY-NOTICES.md"
      },
      "capture": "none",
      "live": "none",
      "evidence_scope": "Named tests exercise the shared adapter/auth contract and synthetic fixtures, not live service availability or entitlement for this model. Cancellation includes terminal-failure or process-shutdown cleanup where named; client-disconnect coverage is separately identifiable from the cited test."
    },
    {
      "provider": "command-code",
      "auth": "command_code_oauth",
      "model": "meta/muse-spark-1.1",
      "wire": "https://api.commandcode.ai/alpha/generate",
      "scenarios": {
        "normal": {
          "status": "covered",
          "evidence": "src/adapters/command_code/router_tests.rs#command_code_tracer_unary"
        },
        "stream": {
          "status": "covered",
          "evidence": "src/adapters/command_code/router_tests.rs#command_code_tracer_response_incremental_and_unary"
        },
        "tools": {
          "status": "covered",
          "evidence": "tests/command_code_translate.rs#command_code_translate_tools_images_orphans_and_ordering"
        },
        "terminal": {
          "status": "covered",
          "evidence": "src/adapters/command_code/router_tests.rs#command_code_tracer_terminal_grammar"
        },
        "malformed": {
          "status": "covered",
          "evidence": "tests/command_code_translate.rs#command_code_translate_envelope_defaults_and_rejections"
        },
        "truncation": {
          "status": "covered",
          "evidence": "src/adapters/command_code/router_tests.rs#command_code_tracer_response_errors_never_emit_success"
        },
        "auth": {
          "status": "covered",
          "evidence": "src/auth/command_code/tests/matrix.rs#command_code_matrix_auth_shapes_file_invariance_and_byte_bounds"
        },
        "cancellation": {
          "status": "covered",
          "evidence": "src/adapters/command_code/router_tests/lifetime.rs#command_code_lifetime_cancel_mid_body_streaming"
        },
        "retry": {
          "status": "covered",
          "evidence": "src/adapters/command_code/router_tests/replay.rs#command_code_lifetime_replay_post_send_timeout_once"
        }
      },
      "model_evidence": "tests/command_code_translate.rs#command_code_translate_effort_exact_table",
      "provenance": {
        "class": "source",
        "repository": "opencodex",
        "revision": "055c3ecf0de6c35f59195fc434d6b08525182b7f",
        "inspected": "2026-09-08",
        "sanitization": "sanitized: exact model/effort tuples only; translated material noticed in THIRD-PARTY-NOTICES.md"
      },
      "capture": "none",
      "live": "none",
      "evidence_scope": "Named tests exercise the shared adapter/auth contract and synthetic fixtures, not live service availability or entitlement for this model. Cancellation includes terminal-failure or process-shutdown cleanup where named; client-disconnect coverage is separately identifiable from the cited test."
    },
    {
      "provider": "command-code",
      "auth": "command_code_oauth",
      "model": "meta/muse-spark-1.2",
      "wire": "https://api.commandcode.ai/alpha/generate",
      "scenarios": {
        "normal": {
          "status": "covered",
          "evidence": "src/adapters/command_code/router_tests.rs#command_code_tracer_unary"
        },
        "stream": {
          "status": "covered",
          "evidence": "src/adapters/command_code/router_tests.rs#command_code_tracer_response_incremental_and_unary"
        },
        "tools": {
          "status": "covered",
          "evidence": "tests/command_code_translate.rs#command_code_translate_tools_images_orphans_and_ordering"
        },
        "terminal": {
          "status": "covered",
          "evidence": "src/adapters/command_code/router_tests.rs#command_code_tracer_terminal_grammar"
        },
        "malformed": {
          "status": "covered",
          "evidence": "tests/command_code_translate.rs#command_code_translate_envelope_defaults_and_rejections"
        },
        "truncation": {
          "status": "covered",
          "evidence": "src/adapters/command_code/router_tests.rs#command_code_tracer_response_errors_never_emit_success"
        },
        "auth": {
          "status": "covered",
          "evidence": "src/auth/command_code/tests/matrix.rs#command_code_matrix_auth_shapes_file_invariance_and_byte_bounds"
        },
        "cancellation": {
          "status": "covered",
          "evidence": "src/adapters/command_code/router_tests/lifetime.rs#command_code_lifetime_cancel_mid_body_streaming"
        },
        "retry": {
          "status": "covered",
          "evidence": "src/adapters/command_code/router_tests/replay.rs#command_code_lifetime_replay_post_send_timeout_once"
        }
      },
      "model_evidence": "tests/command_code_translate.rs#command_code_translate_effort_exact_table",
      "provenance": {
        "class": "source",
        "repository": "opencodex",
        "revision": "055c3ecf0de6c35f59195fc434d6b08525182b7f",
        "inspected": "2026-09-08",
        "sanitization": "sanitized: exact model/effort tuples only; translated material noticed in THIRD-PARTY-NOTICES.md"
      },
      "capture": "none",
      "live": "none",
      "evidence_scope": "Named tests exercise the shared adapter/auth contract and synthetic fixtures, not live service availability or entitlement for this model. Cancellation includes terminal-failure or process-shutdown cleanup where named; client-disconnect coverage is separately identifiable from the cited test."
    },
    {
      "provider": "command-code",
      "auth": "command_code_oauth",
      "model": "meta/muse-spark-1.2-contributor",
      "wire": "https://api.commandcode.ai/alpha/generate",
      "scenarios": {
        "normal": {
          "status": "covered",
          "evidence": "src/adapters/command_code/router_tests.rs#command_code_tracer_unary"
        },
        "stream": {
          "status": "covered",
          "evidence": "src/adapters/command_code/router_tests.rs#command_code_tracer_response_incremental_and_unary"
        },
        "tools": {
          "status": "covered",
          "evidence": "tests/command_code_translate.rs#command_code_translate_tools_images_orphans_and_ordering"
        },
        "terminal": {
          "status": "covered",
          "evidence": "src/adapters/command_code/router_tests.rs#command_code_tracer_terminal_grammar"
        },
        "malformed": {
          "status": "covered",
          "evidence": "tests/command_code_translate.rs#command_code_translate_envelope_defaults_and_rejections"
        },
        "truncation": {
          "status": "covered",
          "evidence": "src/adapters/command_code/router_tests.rs#command_code_tracer_response_errors_never_emit_success"
        },
        "auth": {
          "status": "covered",
          "evidence": "src/auth/command_code/tests/matrix.rs#command_code_matrix_auth_shapes_file_invariance_and_byte_bounds"
        },
        "cancellation": {
          "status": "covered",
          "evidence": "src/adapters/command_code/router_tests/lifetime.rs#command_code_lifetime_cancel_mid_body_streaming"
        },
        "retry": {
          "status": "covered",
          "evidence": "src/adapters/command_code/router_tests/replay.rs#command_code_lifetime_replay_post_send_timeout_once"
        }
      },
      "model_evidence": "tests/command_code_translate.rs#command_code_translate_effort_exact_table",
      "provenance": {
        "class": "source",
        "repository": "opencodex",
        "revision": "055c3ecf0de6c35f59195fc434d6b08525182b7f",
        "inspected": "2026-09-08",
        "sanitization": "sanitized: exact model/effort tuples only; translated material noticed in THIRD-PARTY-NOTICES.md"
      },
      "capture": "none",
      "live": "none",
      "evidence_scope": "Named tests exercise the shared adapter/auth contract and synthetic fixtures, not live service availability or entitlement for this model. Cancellation includes terminal-failure or process-shutdown cleanup where named; client-disconnect coverage is separately identifiable from the cited test."
    },
    {
      "provider": "command-code",
      "auth": "command_code_oauth",
      "model": "zai-org/GLM-5",
      "wire": "https://api.commandcode.ai/alpha/generate",
      "scenarios": {
        "normal": {
          "status": "covered",
          "evidence": "src/adapters/command_code/router_tests.rs#command_code_tracer_unary"
        },
        "stream": {
          "status": "covered",
          "evidence": "src/adapters/command_code/router_tests.rs#command_code_tracer_response_incremental_and_unary"
        },
        "tools": {
          "status": "covered",
          "evidence": "tests/command_code_translate.rs#command_code_translate_tools_images_orphans_and_ordering"
        },
        "terminal": {
          "status": "covered",
          "evidence": "src/adapters/command_code/router_tests.rs#command_code_tracer_terminal_grammar"
        },
        "malformed": {
          "status": "covered",
          "evidence": "tests/command_code_translate.rs#command_code_translate_envelope_defaults_and_rejections"
        },
        "truncation": {
          "status": "covered",
          "evidence": "src/adapters/command_code/router_tests.rs#command_code_tracer_response_errors_never_emit_success"
        },
        "auth": {
          "status": "covered",
          "evidence": "src/auth/command_code/tests/matrix.rs#command_code_matrix_auth_shapes_file_invariance_and_byte_bounds"
        },
        "cancellation": {
          "status": "covered",
          "evidence": "src/adapters/command_code/router_tests/lifetime.rs#command_code_lifetime_cancel_mid_body_streaming"
        },
        "retry": {
          "status": "covered",
          "evidence": "src/adapters/command_code/router_tests/replay.rs#command_code_lifetime_replay_post_send_timeout_once"
        }
      },
      "model_evidence": "tests/command_code_translate.rs#command_code_translate_effort_exact_table",
      "provenance": {
        "class": "source",
        "repository": "opencodex",
        "revision": "055c3ecf0de6c35f59195fc434d6b08525182b7f",
        "inspected": "2026-09-08",
        "sanitization": "sanitized: exact model/effort tuples only; translated material noticed in THIRD-PARTY-NOTICES.md"
      },
      "capture": "none",
      "live": "none",
      "evidence_scope": "Named tests exercise the shared adapter/auth contract and synthetic fixtures, not live service availability or entitlement for this model. Cancellation includes terminal-failure or process-shutdown cleanup where named; client-disconnect coverage is separately identifiable from the cited test."
    },
    {
      "provider": "command-code",
      "auth": "command_code_oauth",
      "model": "zai-org/GLM-5.1",
      "wire": "https://api.commandcode.ai/alpha/generate",
      "scenarios": {
        "normal": {
          "status": "covered",
          "evidence": "src/adapters/command_code/router_tests.rs#command_code_tracer_unary"
        },
        "stream": {
          "status": "covered",
          "evidence": "src/adapters/command_code/router_tests.rs#command_code_tracer_response_incremental_and_unary"
        },
        "tools": {
          "status": "covered",
          "evidence": "tests/command_code_translate.rs#command_code_translate_tools_images_orphans_and_ordering"
        },
        "terminal": {
          "status": "covered",
          "evidence": "src/adapters/command_code/router_tests.rs#command_code_tracer_terminal_grammar"
        },
        "malformed": {
          "status": "covered",
          "evidence": "tests/command_code_translate.rs#command_code_translate_envelope_defaults_and_rejections"
        },
        "truncation": {
          "status": "covered",
          "evidence": "src/adapters/command_code/router_tests.rs#command_code_tracer_response_errors_never_emit_success"
        },
        "auth": {
          "status": "covered",
          "evidence": "src/auth/command_code/tests/matrix.rs#command_code_matrix_auth_shapes_file_invariance_and_byte_bounds"
        },
        "cancellation": {
          "status": "covered",
          "evidence": "src/adapters/command_code/router_tests/lifetime.rs#command_code_lifetime_cancel_mid_body_streaming"
        },
        "retry": {
          "status": "covered",
          "evidence": "src/adapters/command_code/router_tests/replay.rs#command_code_lifetime_replay_post_send_timeout_once"
        }
      },
      "model_evidence": "tests/command_code_translate.rs#command_code_translate_effort_exact_table",
      "provenance": {
        "class": "source",
        "repository": "opencodex",
        "revision": "055c3ecf0de6c35f59195fc434d6b08525182b7f",
        "inspected": "2026-09-08",
        "sanitization": "sanitized: exact model/effort tuples only; translated material noticed in THIRD-PARTY-NOTICES.md"
      },
      "capture": "none",
      "live": "none",
      "evidence_scope": "Named tests exercise the shared adapter/auth contract and synthetic fixtures, not live service availability or entitlement for this model. Cancellation includes terminal-failure or process-shutdown cleanup where named; client-disconnect coverage is separately identifiable from the cited test."
    },
    {
      "provider": "command-code",
      "auth": "command_code_oauth",
      "model": "zai-org/GLM-5.2",
      "wire": "https://api.commandcode.ai/alpha/generate",
      "scenarios": {
        "normal": {
          "status": "covered",
          "evidence": "src/adapters/command_code/router_tests.rs#command_code_tracer_unary"
        },
        "stream": {
          "status": "covered",
          "evidence": "src/adapters/command_code/router_tests.rs#command_code_tracer_response_incremental_and_unary"
        },
        "tools": {
          "status": "covered",
          "evidence": "tests/command_code_translate.rs#command_code_translate_tools_images_orphans_and_ordering"
        },
        "terminal": {
          "status": "covered",
          "evidence": "src/adapters/command_code/router_tests.rs#command_code_tracer_terminal_grammar"
        },
        "malformed": {
          "status": "covered",
          "evidence": "tests/command_code_translate.rs#command_code_translate_envelope_defaults_and_rejections"
        },
        "truncation": {
          "status": "covered",
          "evidence": "src/adapters/command_code/router_tests.rs#command_code_tracer_response_errors_never_emit_success"
        },
        "auth": {
          "status": "covered",
          "evidence": "src/auth/command_code/tests/matrix.rs#command_code_matrix_auth_shapes_file_invariance_and_byte_bounds"
        },
        "cancellation": {
          "status": "covered",
          "evidence": "src/adapters/command_code/router_tests/lifetime.rs#command_code_lifetime_cancel_mid_body_streaming"
        },
        "retry": {
          "status": "covered",
          "evidence": "src/adapters/command_code/router_tests/replay.rs#command_code_lifetime_replay_post_send_timeout_once"
        }
      },
      "model_evidence": "tests/command_code_translate.rs#command_code_translate_effort_exact_table",
      "provenance": {
        "class": "source",
        "repository": "opencodex",
        "revision": "055c3ecf0de6c35f59195fc434d6b08525182b7f",
        "inspected": "2026-09-08",
        "sanitization": "sanitized: exact model/effort tuples only; translated material noticed in THIRD-PARTY-NOTICES.md"
      },
      "capture": "none",
      "live": "none",
      "evidence_scope": "Named tests exercise the shared adapter/auth contract and synthetic fixtures, not live service availability or entitlement for this model. Cancellation includes terminal-failure or process-shutdown cleanup where named; client-disconnect coverage is separately identifiable from the cited test."
    },
    {
      "provider": "command-code",
      "auth": "command_code_oauth",
      "model": "zai-org/GLM-5.2-Fast",
      "wire": "https://api.commandcode.ai/alpha/generate",
      "scenarios": {
        "normal": {
          "status": "covered",
          "evidence": "src/adapters/command_code/router_tests.rs#command_code_tracer_unary"
        },
        "stream": {
          "status": "covered",
          "evidence": "src/adapters/command_code/router_tests.rs#command_code_tracer_response_incremental_and_unary"
        },
        "tools": {
          "status": "covered",
          "evidence": "tests/command_code_translate.rs#command_code_translate_tools_images_orphans_and_ordering"
        },
        "terminal": {
          "status": "covered",
          "evidence": "src/adapters/command_code/router_tests.rs#command_code_tracer_terminal_grammar"
        },
        "malformed": {
          "status": "covered",
          "evidence": "tests/command_code_translate.rs#command_code_translate_envelope_defaults_and_rejections"
        },
        "truncation": {
          "status": "covered",
          "evidence": "src/adapters/command_code/router_tests.rs#command_code_tracer_response_errors_never_emit_success"
        },
        "auth": {
          "status": "covered",
          "evidence": "src/auth/command_code/tests/matrix.rs#command_code_matrix_auth_shapes_file_invariance_and_byte_bounds"
        },
        "cancellation": {
          "status": "covered",
          "evidence": "src/adapters/command_code/router_tests/lifetime.rs#command_code_lifetime_cancel_mid_body_streaming"
        },
        "retry": {
          "status": "covered",
          "evidence": "src/adapters/command_code/router_tests/replay.rs#command_code_lifetime_replay_post_send_timeout_once"
        }
      },
      "model_evidence": "tests/command_code_translate.rs#command_code_translate_effort_exact_table",
      "provenance": {
        "class": "source",
        "repository": "opencodex",
        "revision": "055c3ecf0de6c35f59195fc434d6b08525182b7f",
        "inspected": "2026-09-08",
        "sanitization": "sanitized: exact model/effort tuples only; translated material noticed in THIRD-PARTY-NOTICES.md"
      },
      "capture": "none",
      "live": "none",
      "evidence_scope": "Named tests exercise the shared adapter/auth contract and synthetic fixtures, not live service availability or entitlement for this model. Cancellation includes terminal-failure or process-shutdown cleanup where named; client-disconnect coverage is separately identifiable from the cited test."
    },
    {
      "provider": "command-code",
      "auth": "command_code_oauth",
      "model": "zai-org/GLM-5.3",
      "wire": "https://api.commandcode.ai/alpha/generate",
      "scenarios": {
        "normal": {
          "status": "covered",
          "evidence": "src/adapters/command_code/router_tests.rs#command_code_tracer_unary"
        },
        "stream": {
          "status": "covered",
          "evidence": "src/adapters/command_code/router_tests.rs#command_code_tracer_response_incremental_and_unary"
        },
        "tools": {
          "status": "covered",
          "evidence": "tests/command_code_translate.rs#command_code_translate_tools_images_orphans_and_ordering"
        },
        "terminal": {
          "status": "covered",
          "evidence": "src/adapters/command_code/router_tests.rs#command_code_tracer_terminal_grammar"
        },
        "malformed": {
          "status": "covered",
          "evidence": "tests/command_code_translate.rs#command_code_translate_envelope_defaults_and_rejections"
        },
        "truncation": {
          "status": "covered",
          "evidence": "src/adapters/command_code/router_tests.rs#command_code_tracer_response_errors_never_emit_success"
        },
        "auth": {
          "status": "covered",
          "evidence": "src/auth/command_code/tests/matrix.rs#command_code_matrix_auth_shapes_file_invariance_and_byte_bounds"
        },
        "cancellation": {
          "status": "covered",
          "evidence": "src/adapters/command_code/router_tests/lifetime.rs#command_code_lifetime_cancel_mid_body_streaming"
        },
        "retry": {
          "status": "covered",
          "evidence": "src/adapters/command_code/router_tests/replay.rs#command_code_lifetime_replay_post_send_timeout_once"
        }
      },
      "model_evidence": "tests/command_code_translate.rs#command_code_translate_effort_exact_table",
      "provenance": {
        "class": "source",
        "repository": "opencodex",
        "revision": "055c3ecf0de6c35f59195fc434d6b08525182b7f",
        "inspected": "2026-09-08",
        "sanitization": "sanitized: exact model/effort tuples only; translated material noticed in THIRD-PARTY-NOTICES.md"
      },
      "capture": "none",
      "live": "none",
      "evidence_scope": "Named tests exercise the shared adapter/auth contract and synthetic fixtures, not live service availability or entitlement for this model. Cancellation includes terminal-failure or process-shutdown cleanup where named; client-disconnect coverage is separately identifiable from the cited test."
    },
    {
      "provider": "cursor",
      "auth": "cursor_oauth",
      "model": "composer-2.5",
      "model_policy": "Wire id composer-2.5; the composer-2.5-fast picker alias sends this id with fast=true metadata. Structured tool-history continuation additionally requires this wire model.",
      "wire": "https://agentn.global.api5.cursor.sh/agent.v1.AgentService/Run",
      "preset_seed": "https://api2.cursor.sh (config seed only; the agent transport pins the wire destination above)",
      "scenarios": {
        "normal": {
          "status": "covered",
          "evidence": "src/adapters/cursor/cancellation_tests.rs#cursor_output_parity_full_router_keeps_reasoning_text_usage_order"
        },
        "stream": {
          "status": "covered",
          "evidence": "src/adapters/cursor/cancellation_tests.rs#cursor_cancellation_release_full_router_headers_stream_and_aggregation"
        },
        "tools": {
          "status": "covered",
          "evidence": "src/adapters/cursor/agent.rs#event_stream_emits_native_tool_call"
        },
        "terminal": {
          "status": "covered",
          "evidence": "src/adapters/cursor/router_parity_tests.rs#cursor_terminal_tracer_full_router_eof_and_terminal"
        },
        "malformed": {
          "status": "covered",
          "evidence": "src/adapters/cursor/protocol_tests.rs#cursor_proto_wire_strict_rejects_nested_argument_coercions"
        },
        "truncation": {
          "status": "covered",
          "evidence": "src/adapters/cursor/agent.rs#terminal_event_truncated_frame_is_error"
        },
        "auth": {
          "status": "covered",
          "evidence": "src/adapters/cursor/protocol_tests.rs#cursor_framing_rejection_end_table_preserves_valid_gzip_and_auth"
        },
        "cancellation": {
          "status": "covered",
          "evidence": "src/adapters/cursor/cancellation_tests.rs#cursor_cancellation_release_full_router_headers_stream_and_aggregation"
        },
        "retry": {
          "status": "covered",
          "evidence": "src/adapters/cursor/router_parity_tests.rs#cursor_failure_classification_full_router_never_fails_over_after_acceptance"
        }
      },
      "provenance": {
        "class": "source",
        "repository": "opencodex",
        "revision": "055c3ecf0de6c35f59195fc434d6b08525182b7f",
        "inspected": "2026-09-07",
        "sanitization": "sanitized: source-derived Cursor schemas and synthetic fixtures; no captured credentials or private content"
      },
      "capture": "none",
      "live": "none",
      "evidence_scope": "Named tests exercise the shared adapter/auth contract and synthetic fixtures, not live service availability or entitlement for this model. Cancellation includes terminal-failure or process-shutdown cleanup where named; client-disconnect coverage is separately identifiable from the cited test."
    },
    {
      "provider": "cursor",
      "auth": "cursor_oauth",
      "model": "cursor:default",
      "model_policy": "Agent-mode prefix routes; 'default' is the wire id for Auto (not 'auto').",
      "wire": "https://agentn.global.api5.cursor.sh/agent.v1.AgentService/Run",
      "preset_seed": "https://api2.cursor.sh (config seed only; the agent transport pins the wire destination above)",
      "scenarios": {
        "normal": {
          "status": "covered",
          "evidence": "src/adapters/cursor/cancellation_tests.rs#cursor_output_parity_full_router_keeps_reasoning_text_usage_order"
        },
        "stream": {
          "status": "covered",
          "evidence": "src/adapters/cursor/cancellation_tests.rs#cursor_cancellation_release_full_router_headers_stream_and_aggregation"
        },
        "tools": {
          "status": "covered",
          "evidence": "src/adapters/cursor/agent.rs#event_stream_emits_native_tool_call"
        },
        "terminal": {
          "status": "covered",
          "evidence": "src/adapters/cursor/router_parity_tests.rs#cursor_terminal_tracer_full_router_eof_and_terminal"
        },
        "malformed": {
          "status": "covered",
          "evidence": "src/adapters/cursor/protocol_tests.rs#cursor_proto_wire_strict_rejects_nested_argument_coercions"
        },
        "truncation": {
          "status": "covered",
          "evidence": "src/adapters/cursor/agent.rs#terminal_event_truncated_frame_is_error"
        },
        "auth": {
          "status": "covered",
          "evidence": "src/adapters/cursor/protocol_tests.rs#cursor_framing_rejection_end_table_preserves_valid_gzip_and_auth"
        },
        "cancellation": {
          "status": "covered",
          "evidence": "src/adapters/cursor/cancellation_tests.rs#cursor_cancellation_release_full_router_headers_stream_and_aggregation"
        },
        "retry": {
          "status": "covered",
          "evidence": "src/adapters/cursor/router_parity_tests.rs#cursor_failure_classification_full_router_never_fails_over_after_acceptance"
        }
      },
      "provenance": {
        "class": "source",
        "repository": "opencodex",
        "revision": "055c3ecf0de6c35f59195fc434d6b08525182b7f",
        "inspected": "2026-09-07",
        "sanitization": "sanitized: source-derived Cursor schemas and synthetic fixtures; no captured credentials or private content"
      },
      "capture": "none",
      "live": "none",
      "evidence_scope": "Named tests exercise the shared adapter/auth contract and synthetic fixtures, not live service availability or entitlement for this model. Cancellation includes terminal-failure or process-shutdown cleanup where named; client-disconnect coverage is separately identifiable from the cited test."
    },
    {
      "provider": "grok",
      "auth": "xai_oauth",
      "model": "grok-4.5",
      "model_policy": "Client-declared slug forwarded to the Grok CLI proxy; reasoning effort stays opt-in because several grok families 400 on reasoning.effort (docs/m6-xai-provider.md).",
      "wire": "https://cli-chat-proxy.grok.com/v1/responses",
      "scenarios": {
        "normal": {
          "status": "covered",
          "evidence": "tests/responses_translate.rs#translates_plain_text_request"
        },
        "stream": {
          "status": "covered",
          "evidence": "tests/responses_translate.rs#streaming_state_machine_emits_incremental_anthropic_events"
        },
        "tools": {
          "status": "covered",
          "evidence": "tests/responses_translate.rs#grok_forwards_web_search_tool_and_forced_choice"
        },
        "terminal": {
          "status": "covered",
          "evidence": "tests/responses_translate.rs#normal_completion_records_no_backend_error"
        },
        "malformed": {
          "status": "covered",
          "evidence": "tests/responses_translate.rs#maps_upstream_error_statuses"
        },
        "truncation": {
          "status": "covered",
          "evidence": "src/adapters/responses/http.rs#responses_transport_terminal_http_rejects_a_truncated_upstream"
        },
        "auth": {
          "status": "covered",
          "evidence": "src/adapters/responses/error.rs#maps_401_to_xai_auth_message_for_xai_oauth"
        },
        "cancellation": {
          "status": "covered",
          "evidence": "src/adapters/responses/http.rs#streaming_provider_failure_drops_a_pending_upstream_body"
        },
        "retry": {
          "status": "covered",
          "evidence": "tests/retry.rs#responses_path_does_not_retry_transient_status"
        }
      },
      "provenance": {
        "class": "source",
        "repository": "shunt",
        "revision": "4b7b5bae1956a304b0ac5359d8dd61124c6728dc",
        "inspected": "2026-09-09",
        "sanitization": "sanitized: no credentials, account identifiers, or private content"
      },
      "capture": "none",
      "live": "none",
      "evidence_scope": "Named tests exercise the shared adapter/auth contract and synthetic fixtures, not live service availability or entitlement for this model. Cancellation includes terminal-failure or process-shutdown cleanup where named; client-disconnect coverage is separately identifiable from the cited test."
    },
    {
      "provider": "grok",
      "auth": "xai_oauth",
      "model": "grok-4.6",
      "model_policy": "Client-declared slug forwarded to the Grok CLI proxy; reasoning effort stays opt-in because several grok families 400 on reasoning.effort (docs/m6-xai-provider.md).",
      "wire": "https://cli-chat-proxy.grok.com/v1/responses",
      "scenarios": {
        "normal": {
          "status": "covered",
          "evidence": "tests/responses_translate.rs#translates_plain_text_request"
        },
        "stream": {
          "status": "covered",
          "evidence": "tests/responses_translate.rs#streaming_state_machine_emits_incremental_anthropic_events"
        },
        "tools": {
          "status": "covered",
          "evidence": "tests/responses_translate.rs#grok_forwards_web_search_tool_and_forced_choice"
        },
        "terminal": {
          "status": "covered",
          "evidence": "tests/responses_translate.rs#normal_completion_records_no_backend_error"
        },
        "malformed": {
          "status": "covered",
          "evidence": "tests/responses_translate.rs#maps_upstream_error_statuses"
        },
        "truncation": {
          "status": "covered",
          "evidence": "src/adapters/responses/http.rs#responses_transport_terminal_http_rejects_a_truncated_upstream"
        },
        "auth": {
          "status": "covered",
          "evidence": "src/adapters/responses/error.rs#maps_401_to_xai_auth_message_for_xai_oauth"
        },
        "cancellation": {
          "status": "covered",
          "evidence": "src/adapters/responses/http.rs#streaming_provider_failure_drops_a_pending_upstream_body"
        },
        "retry": {
          "status": "covered",
          "evidence": "tests/retry.rs#responses_path_does_not_retry_transient_status"
        }
      },
      "provenance": {
        "class": "source",
        "repository": "shunt",
        "revision": "4b7b5bae1956a304b0ac5359d8dd61124c6728dc",
        "inspected": "2026-09-09",
        "sanitization": "sanitized: no credentials, account identifiers, or private content"
      },
      "capture": "none",
      "live": "none",
      "evidence_scope": "Named tests exercise the shared adapter/auth contract and synthetic fixtures, not live service availability or entitlement for this model. Cancellation includes terminal-failure or process-shutdown cleanup where named; client-disconnect coverage is separately identifiable from the cited test."
    },
    {
      "provider": "grok",
      "auth": "xai_oauth",
      "model": "grok-build-0.1",
      "model_policy": "Client-declared slug forwarded to the Grok CLI proxy; reasoning effort stays opt-in because several grok families 400 on reasoning.effort (docs/m6-xai-provider.md).",
      "wire": "https://cli-chat-proxy.grok.com/v1/responses",
      "scenarios": {
        "normal": {
          "status": "covered",
          "evidence": "tests/responses_translate.rs#translates_plain_text_request"
        },
        "stream": {
          "status": "covered",
          "evidence": "tests/responses_translate.rs#streaming_state_machine_emits_incremental_anthropic_events"
        },
        "tools": {
          "status": "covered",
          "evidence": "tests/responses_translate.rs#grok_forwards_web_search_tool_and_forced_choice"
        },
        "terminal": {
          "status": "covered",
          "evidence": "tests/responses_translate.rs#normal_completion_records_no_backend_error"
        },
        "malformed": {
          "status": "covered",
          "evidence": "tests/responses_translate.rs#maps_upstream_error_statuses"
        },
        "truncation": {
          "status": "covered",
          "evidence": "src/adapters/responses/http.rs#responses_transport_terminal_http_rejects_a_truncated_upstream"
        },
        "auth": {
          "status": "covered",
          "evidence": "src/adapters/responses/error.rs#maps_401_to_xai_auth_message_for_xai_oauth"
        },
        "cancellation": {
          "status": "covered",
          "evidence": "src/adapters/responses/http.rs#streaming_provider_failure_drops_a_pending_upstream_body"
        },
        "retry": {
          "status": "covered",
          "evidence": "tests/retry.rs#responses_path_does_not_retry_transient_status"
        }
      },
      "provenance": {
        "class": "source",
        "repository": "shunt",
        "revision": "4b7b5bae1956a304b0ac5359d8dd61124c6728dc",
        "inspected": "2026-09-09",
        "sanitization": "sanitized: no credentials, account identifiers, or private content"
      },
      "capture": "none",
      "live": "none",
      "evidence_scope": "Named tests exercise the shared adapter/auth contract and synthetic fixtures, not live service availability or entitlement for this model. Cancellation includes terminal-failure or process-shutdown cleanup where named; client-disconnect coverage is separately identifiable from the cited test."
    },
    {
      "provider": "kimi",
      "auth": "api_key",
      "model": "kimi-k2.7-code",
      "wire": "https://api.moonshot.ai/anthropic/v1/messages",
      "scenarios": {
        "normal": {
          "status": "covered",
          "evidence": "tests/passthrough.rs#messages_forwards_anthropic_headers_verbatim_and_preserves_query"
        },
        "stream": {
          "status": "covered",
          "evidence": "tests/passthrough.rs#relays_sse_response_with_content_type_preserved"
        },
        "tools": {
          "status": "covered",
          "evidence": "tests/inbound_anthropic_translation.rs#enabled_collaboration_translates_and_restores_json_tool_call"
        },
        "terminal": {
          "status": "covered",
          "evidence": "tests/inbound_anthropic_translation.rs#response_premature_stream_eof_is_failed_not_completed"
        },
        "malformed": {
          "status": "covered",
          "evidence": "tests/passthrough.rs#messages_rejects_duplicate_top_level_model_fields"
        },
        "truncation": {
          "status": "covered",
          "evidence": "tests/failover.rs#redispatch_gate_client_visible_truncated_body_stops_without_replaying_turn"
        },
        "auth": {
          "status": "covered",
          "evidence": "src/auth/mod.rs#api_key_provider_requires_env_var"
        },
        "cancellation": {
          "status": "covered",
          "evidence": "tests/passthrough.rs#response_drop_releases_upstream_work_and_global_capacity"
        },
        "retry": {
          "status": "covered",
          "evidence": "tests/retry.rs#messages_path_does_not_retry_transient_status_despite_enabled_policy"
        }
      },
      "provenance": {
        "class": "source",
        "repository": "shunt",
        "revision": "4b7b5bae1956a304b0ac5359d8dd61124c6728dc",
        "inspected": "2026-09-09",
        "sanitization": "sanitized: no credentials, account identifiers, or private content"
      },
      "capture": "none",
      "live": "none",
      "evidence_scope": "Named tests exercise the shared adapter/auth contract and synthetic fixtures, not live service availability or entitlement for this model. Cancellation includes terminal-failure or process-shutdown cleanup where named; client-disconnect coverage is separately identifiable from the cited test."
    },
    {
      "provider": "kimi",
      "auth": "api_key",
      "model": "kimi-k3",
      "wire": "https://api.moonshot.ai/anthropic/v1/messages",
      "scenarios": {
        "normal": {
          "status": "covered",
          "evidence": "tests/passthrough.rs#messages_forwards_anthropic_headers_verbatim_and_preserves_query"
        },
        "stream": {
          "status": "covered",
          "evidence": "tests/passthrough.rs#relays_sse_response_with_content_type_preserved"
        },
        "tools": {
          "status": "covered",
          "evidence": "tests/inbound_anthropic_translation.rs#enabled_collaboration_translates_and_restores_json_tool_call"
        },
        "terminal": {
          "status": "covered",
          "evidence": "tests/inbound_anthropic_translation.rs#response_premature_stream_eof_is_failed_not_completed"
        },
        "malformed": {
          "status": "covered",
          "evidence": "tests/passthrough.rs#messages_rejects_duplicate_top_level_model_fields"
        },
        "truncation": {
          "status": "covered",
          "evidence": "tests/failover.rs#redispatch_gate_client_visible_truncated_body_stops_without_replaying_turn"
        },
        "auth": {
          "status": "covered",
          "evidence": "src/auth/mod.rs#api_key_provider_requires_env_var"
        },
        "cancellation": {
          "status": "covered",
          "evidence": "tests/passthrough.rs#response_drop_releases_upstream_work_and_global_capacity"
        },
        "retry": {
          "status": "covered",
          "evidence": "tests/retry.rs#messages_path_does_not_retry_transient_status_despite_enabled_policy"
        }
      },
      "provenance": {
        "class": "source",
        "repository": "shunt",
        "revision": "4b7b5bae1956a304b0ac5359d8dd61124c6728dc",
        "inspected": "2026-09-09",
        "sanitization": "sanitized: no credentials, account identifiers, or private content"
      },
      "capture": "none",
      "live": "none",
      "evidence_scope": "Named tests exercise the shared adapter/auth contract and synthetic fixtures, not live service availability or entitlement for this model. Cancellation includes terminal-failure or process-shutdown cleanup where named; client-disconnect coverage is separately identifiable from the cited test."
    },
    {
      "provider": "minimax-cn",
      "auth": "api_key",
      "model": "MiniMax-M3",
      "model_policy": "Client-declared id; Anthropic-Messages-compatible MiniMax China endpoint.",
      "wire": "https://api.minimax.cn/anthropic/v1/messages",
      "scenarios": {
        "normal": {
          "status": "covered",
          "evidence": "tests/passthrough.rs#messages_forwards_anthropic_headers_verbatim_and_preserves_query"
        },
        "stream": {
          "status": "covered",
          "evidence": "tests/passthrough.rs#relays_sse_response_with_content_type_preserved"
        },
        "tools": {
          "status": "covered",
          "evidence": "tests/inbound_anthropic_translation.rs#enabled_collaboration_translates_and_restores_json_tool_call"
        },
        "terminal": {
          "status": "covered",
          "evidence": "tests/inbound_anthropic_translation.rs#response_premature_stream_eof_is_failed_not_completed"
        },
        "malformed": {
          "status": "covered",
          "evidence": "tests/passthrough.rs#messages_rejects_duplicate_top_level_model_fields"
        },
        "truncation": {
          "status": "covered",
          "evidence": "tests/failover.rs#redispatch_gate_client_visible_truncated_body_stops_without_replaying_turn"
        },
        "auth": {
          "status": "covered",
          "evidence": "src/auth/mod.rs#api_key_provider_requires_env_var"
        },
        "cancellation": {
          "status": "covered",
          "evidence": "tests/passthrough.rs#response_drop_releases_upstream_work_and_global_capacity"
        },
        "retry": {
          "status": "covered",
          "evidence": "tests/retry.rs#messages_path_does_not_retry_transient_status_despite_enabled_policy"
        }
      },
      "provenance": {
        "class": "source",
        "repository": "shunt",
        "revision": "4b7b5bae1956a304b0ac5359d8dd61124c6728dc",
        "inspected": "2026-09-09",
        "sanitization": "sanitized: no credentials, account identifiers, or private content"
      },
      "capture": "none",
      "live": "none",
      "evidence_scope": "Named tests exercise the shared adapter/auth contract and synthetic fixtures, not live service availability or entitlement for this model. Cancellation includes terminal-failure or process-shutdown cleanup where named; client-disconnect coverage is separately identifiable from the cited test."
    },
    {
      "provider": "openai",
      "auth": "api_key",
      "model": "gpt-5.4",
      "model_policy": "Client-declared id forwarded to the OpenAI Responses endpoint; no per-model support claim.",
      "wire": "https://api.openai.com/v1/responses",
      "scenarios": {
        "normal": {
          "status": "covered",
          "evidence": "tests/responses_translate.rs#translates_plain_text_request"
        },
        "stream": {
          "status": "covered",
          "evidence": "tests/responses_translate.rs#streaming_state_machine_emits_incremental_anthropic_events"
        },
        "tools": {
          "status": "covered",
          "evidence": "tests/responses_translate.rs#translates_tools_and_tool_choice_variants"
        },
        "terminal": {
          "status": "covered",
          "evidence": "tests/responses_translate.rs#normal_completion_records_no_backend_error"
        },
        "malformed": {
          "status": "covered",
          "evidence": "tests/responses_translate.rs#maps_upstream_error_statuses"
        },
        "truncation": {
          "status": "covered",
          "evidence": "src/adapters/responses/http.rs#responses_transport_terminal_http_rejects_a_truncated_upstream"
        },
        "auth": {
          "status": "covered",
          "evidence": "src/auth/mod.rs#api_key_provider_requires_env_var"
        },
        "cancellation": {
          "status": "covered",
          "evidence": "src/adapters/responses/http.rs#streaming_provider_failure_drops_a_pending_upstream_body"
        },
        "retry": {
          "status": "covered",
          "evidence": "tests/retry.rs#responses_path_does_not_retry_transient_status"
        }
      },
      "provenance": {
        "class": "source",
        "repository": "shunt",
        "revision": "4b7b5bae1956a304b0ac5359d8dd61124c6728dc",
        "inspected": "2026-09-09",
        "sanitization": "sanitized: no credentials, account identifiers, or private content"
      },
      "capture": "none",
      "live": "none",
      "evidence_scope": "Named tests exercise the shared adapter/auth contract and synthetic fixtures, not live service availability or entitlement for this model. Cancellation includes terminal-failure or process-shutdown cleanup where named; client-disconnect coverage is separately identifiable from the cited test."
    },
    {
      "provider": "vercel-anthropic",
      "auth": "api_key",
      "model": "anthropic/claude-opus-4.8",
      "model_policy": "Exact route example in site/src/content/docs/providers/vercel-ai-gateway.md; not a catalog-wide or live support claim.",
      "wire": "https://ai-gateway.vercel.sh/v1/messages",
      "scenarios": {
        "normal": {
          "status": "covered",
          "evidence": "tests/passthrough.rs#vercel_anthropic_manual_route_injects_only_selected_x_api_key"
        },
        "stream": {
          "status": "covered",
          "evidence": "tests/passthrough.rs#vercel_anthropic_sse_arrives_before_the_upstream_terminal"
        },
        "tools": {
          "status": "covered",
          "evidence": "tests/inbound_anthropic_translation.rs#enabled_collaboration_translates_and_restores_json_tool_call"
        },
        "terminal": {
          "status": "covered",
          "evidence": "tests/inbound_anthropic_translation.rs#response_premature_stream_eof_is_failed_not_completed"
        },
        "malformed": {
          "status": "covered",
          "evidence": "tests/passthrough.rs#messages_rejects_duplicate_top_level_model_fields"
        },
        "truncation": {
          "status": "covered",
          "evidence": "tests/failover.rs#redispatch_gate_client_visible_truncated_body_stops_without_replaying_turn"
        },
        "auth": {
          "status": "covered",
          "evidence": "tests/passthrough.rs#vercel_anthropic_missing_key_is_redacted_and_never_dispatched"
        },
        "cancellation": {
          "status": "covered",
          "evidence": "tests/passthrough.rs#response_drop_releases_upstream_work_and_global_capacity"
        },
        "retry": {
          "status": "covered",
          "evidence": "tests/passthrough.rs#vercel_anthropic_provider_errors_are_relayed_without_retry"
        }
      },
      "provenance": {
        "class": "source",
        "repository": "shunt",
        "revision": "4b7b5bae1956a304b0ac5359d8dd61124c6728dc",
        "inspected": "2026-09-09",
        "sanitization": "sanitized: no credentials, account identifiers, or private content"
      },
      "capture": "none",
      "live": "none",
      "evidence_scope": "Named tests exercise the shared adapter/auth contract and synthetic fixtures, not live service availability or entitlement for this model. Cancellation includes terminal-failure or process-shutdown cleanup where named; client-disconnect coverage is separately identifiable from the cited test."
    },
    {
      "provider": "xai",
      "auth": "api_key",
      "model": "grok-4.5",
      "model_policy": "Client-declared slug forwarded to the xAI developer API; no per-model support claim (grok-4.6/grok-4.5 live re-verified 2026-08 per docs/m6-xai-provider.md, recorded there, not here).",
      "wire": "https://api.x.ai/v1/responses",
      "scenarios": {
        "normal": {
          "status": "covered",
          "evidence": "tests/responses_translate.rs#translates_plain_text_request"
        },
        "stream": {
          "status": "covered",
          "evidence": "tests/responses_translate.rs#streaming_state_machine_emits_incremental_anthropic_events"
        },
        "tools": {
          "status": "covered",
          "evidence": "tests/responses_translate.rs#xai_drops_web_search_tool_and_downgrades_forced_choice"
        },
        "terminal": {
          "status": "covered",
          "evidence": "tests/responses_translate.rs#normal_completion_records_no_backend_error"
        },
        "malformed": {
          "status": "covered",
          "evidence": "tests/responses_translate.rs#maps_upstream_error_statuses"
        },
        "truncation": {
          "status": "covered",
          "evidence": "src/adapters/responses/http.rs#responses_transport_terminal_http_rejects_a_truncated_upstream"
        },
        "auth": {
          "status": "covered",
          "evidence": "src/auth/mod.rs#api_key_provider_requires_env_var"
        },
        "cancellation": {
          "status": "covered",
          "evidence": "src/adapters/responses/http.rs#streaming_provider_failure_drops_a_pending_upstream_body"
        },
        "retry": {
          "status": "covered",
          "evidence": "tests/retry.rs#responses_path_does_not_retry_transient_status"
        }
      },
      "provenance": {
        "class": "source",
        "repository": "shunt",
        "revision": "4b7b5bae1956a304b0ac5359d8dd61124c6728dc",
        "inspected": "2026-09-09",
        "sanitization": "sanitized: no credentials, account identifiers, or private content"
      },
      "capture": "none",
      "live": "none",
      "evidence_scope": "Named tests exercise the shared adapter/auth contract and synthetic fixtures, not live service availability or entitlement for this model. Cancellation includes terminal-failure or process-shutdown cleanup where named; client-disconnect coverage is separately identifiable from the cited test."
    },
    {
      "provider": "xai",
      "auth": "api_key",
      "model": "grok-4.6",
      "model_policy": "Client-declared slug forwarded to the xAI developer API; no per-model support claim (grok-4.6/grok-4.5 live re-verified 2026-08 per docs/m6-xai-provider.md, recorded there, not here).",
      "wire": "https://api.x.ai/v1/responses",
      "scenarios": {
        "normal": {
          "status": "covered",
          "evidence": "tests/responses_translate.rs#translates_plain_text_request"
        },
        "stream": {
          "status": "covered",
          "evidence": "tests/responses_translate.rs#streaming_state_machine_emits_incremental_anthropic_events"
        },
        "tools": {
          "status": "covered",
          "evidence": "tests/responses_translate.rs#xai_drops_web_search_tool_and_downgrades_forced_choice"
        },
        "terminal": {
          "status": "covered",
          "evidence": "tests/responses_translate.rs#normal_completion_records_no_backend_error"
        },
        "malformed": {
          "status": "covered",
          "evidence": "tests/responses_translate.rs#maps_upstream_error_statuses"
        },
        "truncation": {
          "status": "covered",
          "evidence": "src/adapters/responses/http.rs#responses_transport_terminal_http_rejects_a_truncated_upstream"
        },
        "auth": {
          "status": "covered",
          "evidence": "src/auth/mod.rs#api_key_provider_requires_env_var"
        },
        "cancellation": {
          "status": "covered",
          "evidence": "src/adapters/responses/http.rs#streaming_provider_failure_drops_a_pending_upstream_body"
        },
        "retry": {
          "status": "covered",
          "evidence": "tests/retry.rs#responses_path_does_not_retry_transient_status"
        }
      },
      "provenance": {
        "class": "source",
        "repository": "shunt",
        "revision": "4b7b5bae1956a304b0ac5359d8dd61124c6728dc",
        "inspected": "2026-09-09",
        "sanitization": "sanitized: no credentials, account identifiers, or private content"
      },
      "capture": "none",
      "live": "none",
      "evidence_scope": "Named tests exercise the shared adapter/auth contract and synthetic fixtures, not live service availability or entitlement for this model. Cancellation includes terminal-failure or process-shutdown cleanup where named; client-disconnect coverage is separately identifiable from the cited test."
    },
    {
      "provider": "xai",
      "auth": "api_key",
      "model": "grok-build-0.1",
      "model_policy": "Client-declared slug forwarded to the xAI developer API; no per-model support claim (grok-4.6/grok-4.5 live re-verified 2026-08 per docs/m6-xai-provider.md, recorded there, not here).",
      "wire": "https://api.x.ai/v1/responses",
      "scenarios": {
        "normal": {
          "status": "covered",
          "evidence": "tests/responses_translate.rs#translates_plain_text_request"
        },
        "stream": {
          "status": "covered",
          "evidence": "tests/responses_translate.rs#streaming_state_machine_emits_incremental_anthropic_events"
        },
        "tools": {
          "status": "covered",
          "evidence": "tests/responses_translate.rs#xai_drops_web_search_tool_and_downgrades_forced_choice"
        },
        "terminal": {
          "status": "covered",
          "evidence": "tests/responses_translate.rs#normal_completion_records_no_backend_error"
        },
        "malformed": {
          "status": "covered",
          "evidence": "tests/responses_translate.rs#maps_upstream_error_statuses"
        },
        "truncation": {
          "status": "covered",
          "evidence": "src/adapters/responses/http.rs#responses_transport_terminal_http_rejects_a_truncated_upstream"
        },
        "auth": {
          "status": "covered",
          "evidence": "src/auth/mod.rs#api_key_provider_requires_env_var"
        },
        "cancellation": {
          "status": "covered",
          "evidence": "src/adapters/responses/http.rs#streaming_provider_failure_drops_a_pending_upstream_body"
        },
        "retry": {
          "status": "covered",
          "evidence": "tests/retry.rs#responses_path_does_not_retry_transient_status"
        }
      },
      "provenance": {
        "class": "source",
        "repository": "shunt",
        "revision": "4b7b5bae1956a304b0ac5359d8dd61124c6728dc",
        "inspected": "2026-09-09",
        "sanitization": "sanitized: no credentials, account identifiers, or private content"
      },
      "capture": "none",
      "live": "none",
      "evidence_scope": "Named tests exercise the shared adapter/auth contract and synthetic fixtures, not live service availability or entitlement for this model. Cancellation includes terminal-failure or process-shutdown cleanup where named; client-disconnect coverage is separately identifiable from the cited test."
    },
    {
      "provider": "zhipu",
      "auth": "api_key",
      "model": "glm-5.3",
      "model_policy": "Client-declared id; Anthropic-Messages-compatible Zhipu endpoint.",
      "wire": "https://open.bigmodel.cn/api/anthropic/v1/messages",
      "scenarios": {
        "normal": {
          "status": "covered",
          "evidence": "tests/passthrough.rs#messages_forwards_anthropic_headers_verbatim_and_preserves_query"
        },
        "stream": {
          "status": "covered",
          "evidence": "tests/passthrough.rs#relays_sse_response_with_content_type_preserved"
        },
        "tools": {
          "status": "covered",
          "evidence": "tests/inbound_anthropic_translation.rs#enabled_collaboration_translates_and_restores_json_tool_call"
        },
        "terminal": {
          "status": "covered",
          "evidence": "tests/inbound_anthropic_translation.rs#response_premature_stream_eof_is_failed_not_completed"
        },
        "malformed": {
          "status": "covered",
          "evidence": "tests/passthrough.rs#messages_rejects_duplicate_top_level_model_fields"
        },
        "truncation": {
          "status": "covered",
          "evidence": "tests/failover.rs#redispatch_gate_client_visible_truncated_body_stops_without_replaying_turn"
        },
        "auth": {
          "status": "covered",
          "evidence": "src/auth/mod.rs#api_key_provider_requires_env_var"
        },
        "cancellation": {
          "status": "covered",
          "evidence": "tests/passthrough.rs#response_drop_releases_upstream_work_and_global_capacity"
        },
        "retry": {
          "status": "covered",
          "evidence": "tests/retry.rs#messages_path_does_not_retry_transient_status_despite_enabled_policy"
        }
      },
      "provenance": {
        "class": "source",
        "repository": "shunt",
        "revision": "4b7b5bae1956a304b0ac5359d8dd61124c6728dc",
        "inspected": "2026-09-09",
        "sanitization": "sanitized: no credentials, account identifiers, or private content"
      },
      "capture": "none",
      "live": "none",
      "evidence_scope": "Named tests exercise the shared adapter/auth contract and synthetic fixtures, not live service availability or entitlement for this model. Cancellation includes terminal-failure or process-shutdown cleanup where named; client-disconnect coverage is separately identifiable from the cited test."
    },
    {
      "provider": "zhipu",
      "auth": "api_key",
      "model": "glm-5.3-flash",
      "model_policy": "Client-declared id; Anthropic-Messages-compatible Zhipu endpoint.",
      "wire": "https://open.bigmodel.cn/api/anthropic/v1/messages",
      "scenarios": {
        "normal": {
          "status": "covered",
          "evidence": "tests/passthrough.rs#messages_forwards_anthropic_headers_verbatim_and_preserves_query"
        },
        "stream": {
          "status": "covered",
          "evidence": "tests/passthrough.rs#relays_sse_response_with_content_type_preserved"
        },
        "tools": {
          "status": "covered",
          "evidence": "tests/inbound_anthropic_translation.rs#enabled_collaboration_translates_and_restores_json_tool_call"
        },
        "terminal": {
          "status": "covered",
          "evidence": "tests/inbound_anthropic_translation.rs#response_premature_stream_eof_is_failed_not_completed"
        },
        "malformed": {
          "status": "covered",
          "evidence": "tests/passthrough.rs#messages_rejects_duplicate_top_level_model_fields"
        },
        "truncation": {
          "status": "covered",
          "evidence": "tests/failover.rs#redispatch_gate_client_visible_truncated_body_stops_without_replaying_turn"
        },
        "auth": {
          "status": "covered",
          "evidence": "src/auth/mod.rs#api_key_provider_requires_env_var"
        },
        "cancellation": {
          "status": "covered",
          "evidence": "tests/passthrough.rs#response_drop_releases_upstream_work_and_global_capacity"
        },
        "retry": {
          "status": "covered",
          "evidence": "tests/retry.rs#messages_path_does_not_retry_transient_status_despite_enabled_policy"
        }
      },
      "provenance": {
        "class": "source",
        "repository": "shunt",
        "revision": "4b7b5bae1956a304b0ac5359d8dd61124c6728dc",
        "inspected": "2026-09-09",
        "sanitization": "sanitized: no credentials, account identifiers, or private content"
      },
      "capture": "none",
      "live": "none",
      "evidence_scope": "Named tests exercise the shared adapter/auth contract and synthetic fixtures, not live service availability or entitlement for this model. Cancellation includes terminal-failure or process-shutdown cleanup where named; client-disconnect coverage is separately identifiable from the cited test."
    }
  ],
  "go": {
    "admitted": [],
    "policy": "OpenCode Go admits zero tuples per D-02. Future promotion requires exact hermetic conformance plus credential-safe captured or live verification under the existing evidence gate; this ledger grants no admission."
  },
  "unbound_contracts": [
    {
      "provider": "antigravity-cli",
      "auth": "none",
      "model_policy": "Deprecated local agy subprocess transport; arbitrary code execution documented in README.",
      "wire": "local://agy-subprocess",
      "scenarios": {
        "normal": {
          "status": "covered",
          "evidence": "tests/antigravity_process.rs#non_streaming_turn_returns_the_translated_message"
        },
        "stream": {
          "status": "covered",
          "evidence": "tests/antigravity_process.rs#streaming_turn_translates_stub_events_to_sse"
        },
        "tools": {
          "status": "not_applicable",
          "rationale": "The agy subprocess can never return a tool_use block; tool-requesting inputs are refused with 400 per the documented contract."
        },
        "terminal": {
          "status": "covered",
          "evidence": "tests/antigravity_process.rs#streaming_premature_eof_does_not_hang_and_reports_an_error"
        },
        "malformed": {
          "status": "covered",
          "evidence": "tests/antigravity_translate.rs#test_translator_tolerates_garbage_lines"
        },
        "truncation": {
          "status": "covered",
          "evidence": "tests/antigravity_translate.rs#test_premature_eof_is_not_reported_as_success"
        },
        "auth": {
          "status": "not_applicable",
          "rationale": "Local subprocess uses no stored credential; sandbox and permission semantics are documented in README and enforced by sandbox defaults."
        },
        "cancellation": {
          "status": "covered",
          "evidence": "src/adapters/antigravity/child.rs#terminate_all_groups_sweeps_registered_and_late_arriving_children"
        },
        "retry": {
          "status": "not_applicable",
          "rationale": "The local subprocess transport does not expose HTTP retry or provider failover; its one-process terminal failure is covered by tests/antigravity_process.rs#non_streaming_failure_reports_the_exit_status."
        }
      },
      "provenance": {
        "class": "source",
        "repository": "shunt",
        "revision": "4b7b5bae1956a304b0ac5359d8dd61124c6728dc",
        "inspected": "2026-09-09",
        "sanitization": "sanitized: no credentials, account identifiers, or private content"
      },
      "capture": "none",
      "live": "none",
      "evidence_scope": "Named tests exercise the shared adapter/auth contract and synthetic fixtures, not live service availability or entitlement for this model. Cancellation includes terminal-failure or process-shutdown cleanup where named; client-disconnect coverage is separately identifiable from the cited test.",
      "selection": "No finite model identity promised by this generic or account-catalog contract. Configure an exact account-supported model; no arbitrary-model support is inferred."
    },
    {
      "provider": "commandcode",
      "auth": "api_key",
      "model_policy": "Client-declared id forwarded to the OpenAI-Chat endpoint; no per-model support claim.",
      "wire": "https://api.commandcode.ai/provider/v1/chat/completions",
      "scenarios": {
        "normal": {
          "status": "covered",
          "evidence": "tests/command_code_api_conformance.rs#command_code_api_tracer_unary"
        },
        "stream": {
          "status": "covered",
          "evidence": "tests/command_code_api_conformance.rs#command_code_api_scenarios_stream_terminal"
        },
        "tools": {
          "status": "covered",
          "evidence": "tests/command_code_api_conformance.rs#command_code_api_scenarios_tools_and_long_context"
        },
        "terminal": {
          "status": "covered",
          "evidence": "tests/command_code_api_conformance.rs#command_code_api_scenarios_stream_terminal"
        },
        "malformed": {
          "status": "covered",
          "evidence": "tests/openai_chat_conformance.rs#openai_chat_terminal_malformed_json_rejected"
        },
        "truncation": {
          "status": "covered",
          "evidence": "tests/openai_chat_conformance.rs#openai_chat_tracer_streaming_eof_failclosed"
        },
        "auth": {
          "status": "covered",
          "evidence": "tests/command_code_api_conformance.rs#command_code_api_tracer_auth_error"
        },
        "cancellation": {
          "status": "covered",
          "evidence": "tests/command_code_api_conformance.rs#command_code_api_scenarios_incremental_drop_releases_capacity"
        },
        "retry": {
          "status": "covered",
          "evidence": "tests/retry.rs#non_transient_400_surfaces_without_retry"
        }
      },
      "provenance": {
        "class": "source",
        "repository": "shunt",
        "revision": "4b7b5bae1956a304b0ac5359d8dd61124c6728dc",
        "inspected": "2026-09-09",
        "sanitization": "sanitized: no credentials, account identifiers, or private content"
      },
      "capture": "none",
      "live": "none",
      "evidence_scope": "Named tests exercise the shared adapter/auth contract and synthetic fixtures, not live service availability or entitlement for this model. Cancellation includes terminal-failure or process-shutdown cleanup where named; client-disconnect coverage is separately identifiable from the cited test.",
      "selection": "No finite model identity promised by this generic or account-catalog contract. Configure an exact account-supported model; no arbitrary-model support is inferred."
    },
    {
      "provider": "custom-anthropic",
      "auth": "passthrough",
      "model_policy": "Client-declared id and base URL for any Anthropic-Messages-compatible upstream; no per-model support claim.",
      "wire": "client-declared-base-url/v1/messages",
      "scenarios": {
        "normal": {
          "status": "covered",
          "evidence": "tests/passthrough.rs#messages_forwards_anthropic_headers_verbatim_and_preserves_query"
        },
        "stream": {
          "status": "covered",
          "evidence": "tests/passthrough.rs#relays_sse_response_with_content_type_preserved"
        },
        "tools": {
          "status": "covered",
          "evidence": "tests/inbound_anthropic_translation.rs#enabled_collaboration_translates_and_restores_json_tool_call"
        },
        "terminal": {
          "status": "covered",
          "evidence": "tests/inbound_anthropic_translation.rs#response_premature_stream_eof_is_failed_not_completed"
        },
        "malformed": {
          "status": "covered",
          "evidence": "tests/passthrough.rs#messages_rejects_duplicate_top_level_model_fields"
        },
        "truncation": {
          "status": "covered",
          "evidence": "tests/failover.rs#redispatch_gate_client_visible_truncated_body_stops_without_replaying_turn"
        },
        "auth": {
          "status": "covered",
          "evidence": "tests/passthrough.rs#messages_forwards_incoming_credentials_unchanged"
        },
        "cancellation": {
          "status": "covered",
          "evidence": "tests/passthrough.rs#response_drop_releases_upstream_work_and_global_capacity"
        },
        "retry": {
          "status": "covered",
          "evidence": "tests/retry.rs#messages_path_does_not_retry_transient_status_despite_enabled_policy"
        }
      },
      "provenance": {
        "class": "source",
        "repository": "shunt",
        "revision": "4b7b5bae1956a304b0ac5359d8dd61124c6728dc",
        "inspected": "2026-09-09",
        "sanitization": "sanitized: no credentials, account identifiers, or private content"
      },
      "capture": "none",
      "live": "none",
      "evidence_scope": "Named tests exercise the shared adapter/auth contract and synthetic fixtures, not live service availability or entitlement for this model. Cancellation includes terminal-failure or process-shutdown cleanup where named; client-disconnect coverage is separately identifiable from the cited test.",
      "selection": "No finite model identity promised by this generic or account-catalog contract. Configure an exact account-supported model; no arbitrary-model support is inferred."
    },
    {
      "provider": "custom-openai-chat",
      "auth": "api_key",
      "model_policy": "Client-declared id and base URL for an OpenAI-Chat-Completions-compatible upstream; no vendor-wide or per-model support claim. Vercel's documented Anthropic route is represented separately.",
      "wire": "client-declared-base-url/chat/completions",
      "scenarios": {
        "normal": {
          "status": "covered",
          "evidence": "tests/openai_chat_conformance.rs#openai_chat_translate_wire_exact_body"
        },
        "stream": {
          "status": "covered",
          "evidence": "tests/openai_chat_conformance.rs#openai_chat_tracer_streaming_relay"
        },
        "tools": {
          "status": "covered",
          "evidence": "tests/openai_chat_conformance.rs#openai_chat_translate_wire_tool_pairing"
        },
        "terminal": {
          "status": "covered",
          "evidence": "tests/openai_chat_conformance.rs#openai_chat_terminal_missing_finish_eof_fails"
        },
        "malformed": {
          "status": "covered",
          "evidence": "tests/openai_chat_conformance.rs#openai_chat_terminal_malformed_json_rejected"
        },
        "truncation": {
          "status": "covered",
          "evidence": "tests/openai_chat_conformance.rs#openai_chat_tracer_streaming_eof_failclosed"
        },
        "auth": {
          "status": "covered",
          "evidence": "tests/openai_chat_conformance.rs#openai_chat_auth_redirect_refusal"
        },
        "cancellation": {
          "status": "covered",
          "evidence": "tests/command_code_api_conformance.rs#command_code_api_scenarios_incremental_drop_releases_capacity"
        },
        "retry": {
          "status": "covered",
          "evidence": "tests/retry.rs#non_transient_400_surfaces_without_retry"
        }
      },
      "provenance": {
        "class": "source",
        "repository": "shunt",
        "revision": "4b7b5bae1956a304b0ac5359d8dd61124c6728dc",
        "inspected": "2026-09-09",
        "sanitization": "sanitized: no credentials, account identifiers, or private content"
      },
      "capture": "none",
      "live": "none",
      "evidence_scope": "Named tests exercise the shared adapter/auth contract and synthetic fixtures, not live service availability or entitlement for this model. Cancellation includes terminal-failure or process-shutdown cleanup where named; client-disconnect coverage is separately identifiable from the cited test.",
      "selection": "No finite model identity promised by this generic or account-catalog contract. Configure an exact account-supported model; no arbitrary-model support is inferred."
    },
    {
      "provider": "gemini",
      "auth": "google_oauth",
      "model_policy": "An operator-declared upstream model is carried in the Google Code Assist request envelope; no catalog gate or per-model entitlement claim is inferred from the distinct native Antigravity contract.",
      "wire": "https://cloudcode-pa.googleapis.com/v1internal:{streamGenerateContent|generateContent}",
      "scenarios": {
        "normal": {
          "status": "covered",
          "evidence": "tests/gemini_conformance.rs#gemini_post_done_frames_real_gateway"
        },
        "stream": {
          "status": "covered",
          "evidence": "tests/gemini_conformance.rs#gemini_streaming_framing_accepts_wrapped_crlf_and_authoritative_finish"
        },
        "tools": {
          "status": "covered",
          "evidence": "tests/gemini_conformance.rs#gemini_no_post_header_replay_preserves_tool_call_result_pairing_next_turn"
        },
        "terminal": {
          "status": "covered",
          "evidence": "tests/gemini_conformance.rs#gemini_streaming_framing_embedded_provider_error_is_terminal"
        },
        "malformed": {
          "status": "covered",
          "evidence": "tests/gemini_conformance.rs#gemini_streaming_framing_rejects_malformed_json_once"
        },
        "truncation": {
          "status": "covered",
          "evidence": "tests/gemini_conformance.rs#gemini_streaming_framing_rejects_cut_stream_without_synthetic_success"
        },
        "auth": {
          "status": "covered",
          "evidence": "src/auth/google/auth.rs#revoked_refresh_token_returns_clear_auth_error"
        },
        "cancellation": {
          "status": "covered",
          "evidence": "tests/gemini_conformance.rs#gemini_response_drop_releases_upstream_and_gateway_capacity"
        },
        "retry": {
          "status": "covered",
          "evidence": "tests/gemini_conformance.rs#gemini_identity_retry_never_retries_returned_transient_statuses"
        }
      },
      "provenance": {
        "class": "source",
        "repository": "shunt",
        "revision": "4b7b5bae1956a304b0ac5359d8dd61124c6728dc",
        "inspected": "2026-09-09",
        "sanitization": "sanitized: no credentials, account identifiers, or private content"
      },
      "capture": "none",
      "live": "none",
      "evidence_scope": "Named tests exercise the shared adapter/auth contract and synthetic fixtures, not live service availability or entitlement for this model. Cancellation includes terminal-failure or process-shutdown cleanup where named; client-disconnect coverage is separately identifiable from the cited test.",
      "selection": "No finite model identity promised by this generic or account-catalog contract. Configure an exact account-supported model; no arbitrary-model support is inferred."
    },
    {
      "provider": "kimi-code",
      "auth": "kimi_oauth",
      "model_policy": "Client-declared id; Kimi Code is the subscription OAuth Kimi service, distinct from the metered kimi preset.",
      "wire": "https://api.kimi.com/coding/v1/messages",
      "scenarios": {
        "normal": {
          "status": "covered",
          "evidence": "tests/passthrough.rs#messages_forwards_anthropic_headers_verbatim_and_preserves_query"
        },
        "stream": {
          "status": "covered",
          "evidence": "tests/passthrough.rs#relays_sse_response_with_content_type_preserved"
        },
        "tools": {
          "status": "covered",
          "evidence": "tests/inbound_anthropic_translation.rs#enabled_collaboration_translates_and_restores_json_tool_call"
        },
        "terminal": {
          "status": "covered",
          "evidence": "tests/inbound_anthropic_translation.rs#response_premature_stream_eof_is_failed_not_completed"
        },
        "malformed": {
          "status": "covered",
          "evidence": "tests/passthrough.rs#messages_rejects_duplicate_top_level_model_fields"
        },
        "truncation": {
          "status": "covered",
          "evidence": "tests/failover.rs#redispatch_gate_client_visible_truncated_body_stops_without_replaying_turn"
        },
        "auth": {
          "status": "covered",
          "evidence": "tests/kimi_multi_account.rs#pool_rotates_to_next_account_on_401"
        },
        "cancellation": {
          "status": "covered",
          "evidence": "tests/passthrough.rs#response_drop_releases_upstream_work_and_global_capacity"
        },
        "retry": {
          "status": "covered",
          "evidence": "tests/retry.rs#messages_path_does_not_retry_transient_status_despite_enabled_policy"
        }
      },
      "provenance": {
        "class": "source",
        "repository": "shunt",
        "revision": "4b7b5bae1956a304b0ac5359d8dd61124c6728dc",
        "inspected": "2026-09-09",
        "sanitization": "sanitized: no credentials, account identifiers, or private content"
      },
      "capture": "none",
      "live": "none",
      "evidence_scope": "Named tests exercise the shared adapter/auth contract and synthetic fixtures, not live service availability or entitlement for this model. Cancellation includes terminal-failure or process-shutdown cleanup where named; client-disconnect coverage is separately identifiable from the cited test.",
      "selection": "No finite model identity promised by this generic or account-catalog contract. Configure an exact account-supported model; no arbitrary-model support is inferred."
    }
  ]
}
```

## Provenance classes

- **source** — every row above: derived from tracked repository source at the
  pinned revision, with the per-row inspection date. Command Code subscription rows are
  translated from OpenCodex at `055c3ecf0de6c35f59195fc434d6b08525182b7f`
  (noticed in `THIRD-PARTY-NOTICES.md`, sanitized: exact model/effort tuples
  only).
- **capture** / **live** — separate per-row fields, currently `none`. Phase 16
  live-smoke plans record actual bounded outcomes separately under the D-04–D-06
  budget; a missing record is never inferred from a source-derived pass.
- **static** — visual/build checks are recorded by the phase verification
  plans, never inside this ledger's rows.

The preset seed base URL (for example Cursor's `https://api2.cursor.sh`) is
never the wire destination: each row's `wire` field records the destination the
adapter actually sends to, pinned by the named regression tests cited above.
