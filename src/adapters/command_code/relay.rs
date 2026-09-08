//! The stream/parser/machine tuple owns its upstream; no detached recovery task.
use super::{ndjson::Decoder, semantic};
use crate::{
    adapters::AdapterError,
    model::command_code_response::{CommandCodeMachine, SemanticError, SseEvent},
};
use axum::{
    body::{Body, Bytes},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use futures_util::{Stream, StreamExt};
use std::{convert::Infallible, pin::Pin, time::Duration};

type ByteStream = Pin<Box<dyn Stream<Item = Result<Bytes, reqwest::Error>> + Send>>;

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test(start_paused = true)]
    async fn command_code_bounds_slow_drip_record_deadline() {
        let bytes = futures_util::stream::unfold((), |_| async {
            tokio::time::sleep(Duration::from_secs(10)).await;
            Some((Ok(Bytes::from_static(b" ")), ()))
        });
        let mut relay = Relay::new(Box::pin(bytes), "alias".into(), false);
        let result = tokio::time::timeout(Duration::from_secs(121), relay.next()).await;
        assert!(matches!(result, Ok(Err(ref error)) if error.message == "subscription record deadline exceeded"),
            "slow-drip records must fail by the 120-second record deadline, not merely idle timeout");
    }
}

struct Relay {
    bytes: Option<ByteStream>,
    decoder: Decoder,
    machine: CommandCodeMachine,
    done: bool,
}
impl Relay {
    fn new(bytes: ByteStream, model: String, streaming: bool) -> Self {
        Self {
            bytes: Some(bytes),
            decoder: Decoder::default(),
            machine: if streaming {
                CommandCodeMachine::new_streaming(model)
            } else {
                CommandCodeMachine::new(model)
            },
            done: false,
        }
    }
    async fn next(&mut self) -> Result<Option<Vec<SseEvent>>, SemanticError> {
        if self.done {
            return Ok(None);
        }
        let result = self.read_next().await;
        if result.is_err() {
            self.done = true;
            self.bytes = None;
        }
        result
    }
    async fn read_next(&mut self) -> Result<Option<Vec<SseEvent>>, SemanticError> {
        loop {
            let chunk = self
                .bytes
                .as_mut()
                .expect("active relay owns stream")
                .next()
                .await;
            match chunk {
                Some(Ok(chunk)) => {
                    let records = self.decoder.feed(&chunk).map_err(SemanticError::protocol)?;
                    let mut out = Vec::new();
                    // Validate the complete received batch before yielding its events.
                    // Never emit success followed by error for an already-received duplicate.
                    for record in records {
                        out.extend(self.machine.process_record_checked(&record)?);
                    }
                    if !out.is_empty() {
                        return Ok(Some(out));
                    }
                }
                Some(Err(_)) => {
                    return Err(SemanticError::protocol(
                        "subscription body transport failed",
                    ))
                }
                None => {
                    self.bytes = None;
                    self.done = true;
                    self.decoder.finish().map_err(SemanticError::protocol)?;
                    return self.machine.transport_close_checked().map(Some);
                }
            }
        }
    }
}

fn encode(events: Vec<SseEvent>) -> Bytes {
    let mut bytes = Vec::new();
    for event in events {
        bytes.extend_from_slice(
            format!("event: {}\ndata: {}\n\n", event.event, event.data).as_bytes(),
        );
    }
    Bytes::from(bytes)
}

pub(super) async fn respond(
    response: reqwest::Response,
    model: String,
    streaming: bool,
    keepalive: Duration,
) -> Result<(StatusCode, Response<Body>), AdapterError> {
    let mut relay = Relay::new(Box::pin(response.bytes_stream()), model, streaming);
    if !streaming {
        while relay.next().await.map_err(semantic)?.is_some() {}
        let body = relay.machine.final_ndjson_checked().map_err(semantic)?;
        return Ok((StatusCode::OK, axum::Json(body).into_response()));
    }
    let stream = futures_util::stream::unfold(relay, |mut relay| async move {
        let events = match relay.next().await {
            Ok(Some(events)) => events,
            Ok(None) => return None,
            Err(error) => vec![SseEvent {
                event: "error".into(),
                data: error.body(),
            }],
        };
        Some((Ok::<_, Infallible>(encode(events)), relay))
    });
    let response = Response::builder()
        .status(StatusCode::OK)
        .header("content-type", "text/event-stream; charset=utf-8")
        .header("cache-control", "no-cache")
        .body(Body::from_stream(crate::keepalive::with_pings(
            stream, keepalive,
        )))
        .map_err(|_| super::protocol("failed to build subscription response"))?;
    Ok((StatusCode::OK, response))
}
