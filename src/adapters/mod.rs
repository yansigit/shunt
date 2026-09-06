use std::{future::Future, pin::Pin};

use axum::{
    http::{HeaderMap, StatusCode, Uri},
    response::Response,
};

use crate::{request::RequestBody, routing::Route, server::AppState};

pub mod anthropic;
pub mod antigravity;
pub mod cursor;
pub mod gemini;
pub mod responses;

/// Tie a storm-control [`AdmissionGuard`](crate::accounts::AdmissionGuard) to a
/// relayed response (issue #195). The response body is lazy — for a streaming
/// turn the adapter function returns long before axum drives the SSE bytes to
/// the client — so dropping the guard at return would free the admission slot
/// at roughly time-to-first-byte instead of for the turn's real duration. Moving
/// the guard into the body stream makes it drop when the stream is exhausted or
/// the client disconnects, so `in_flight` counts genuinely concurrent turns.
pub(crate) fn with_admission(
    response: Response,
    admission: Option<crate::accounts::AdmissionGuard>,
) -> Response {
    use futures_util::StreamExt;
    let Some(guard) = admission else {
        return response;
    };
    let (parts, body) = response.into_parts();
    let stream = body.into_data_stream().map(move |chunk| {
        // The `move` closure owns the guard; this reference only forces the
        // capture (a variable the body never touches is not captured at all).
        // The owned guard then drops with the closure when the stream does.
        let _held = &guard;
        chunk
    });
    Response::from_parts(parts, axum::body::Body::from_stream(stream))
}

pub type AdapterResult = Result<(StatusCode, Response), AdapterError>;
pub type AdapterFuture<'a> = Pin<Box<dyn Future<Output = AdapterResult> + Send + 'a>>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AdapterFailure {
    /// The upstream returned this status before the adapter mapped it for the client.
    UpstreamStatus(StatusCode),
    /// The attempt failed before any upstream response headers were received.
    BeforeHeaders,
}

#[derive(Debug)]
pub struct AdapterError {
    pub message: String,
    pub response: Box<Response>,
    /// Additive failover metadata. `None` marks a local adapter/auth/validation
    /// error that must be returned immediately rather than retried elsewhere.
    pub failure: Option<AdapterFailure>,
}

pub(crate) trait Adapter {
    fn forward<'a>(
        &'a self,
        state: AppState,
        route: Route,
        uri: &'a Uri,
        headers: &'a HeaderMap,
        body: RequestBody,
    ) -> AdapterFuture<'a>;
}

#[cfg(test)]
mod tests {
    use std::{convert::Infallible, future::poll_fn, pin::Pin, sync::Arc};

    use axum::body::{Body, HttpBody};
    use futures_util::{stream, StreamExt};

    use super::with_admission;
    use crate::{accounts::AccountPool, config::AccountConfig};

    fn account(name: &str) -> AccountConfig {
        AccountConfig {
            name: name.to_string(),
            ..Default::default()
        }
    }

    #[tokio::test]
    async fn with_admission_holds_slot_until_body_is_consumed() {
        let pool = Arc::new(AccountPool::new());
        let acc = account("a");
        let guard = pool
            .clone()
            .try_admit("codex", &acc, 1, false)
            .expect("first admission");

        let response = with_admission(
            axum::response::Response::new(Body::from("data: chunk\n\n")),
            Some(guard),
        );

        // The slot stays occupied while the wrapped body is still pending —
        // the guard must not drop at `with_admission`'s return.
        assert!(
            pool.clone().try_admit("codex", &acc, 1, false).is_none(),
            "slot should be held while the response body is unread"
        );

        let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("body drives to completion");
        assert_eq!(bytes.as_ref(), b"data: chunk\n\n");

        let reguard = pool.clone().try_admit("codex", &acc, 1, false);
        assert!(
            reguard.is_some(),
            "slot should free once the body stream is exhausted"
        );
    }

    #[tokio::test]
    async fn response_drop_releases_account_admission_after_partial_stream() {
        let pool = Arc::new(AccountPool::new());
        let acc = account("a");
        let guard = pool
            .clone()
            .try_admit("codex", &acc, 1, false)
            .expect("first admission");

        let response = with_admission(
            axum::response::Response::new(Body::from_stream(
                stream::once(async { Ok::<_, Infallible>("data: chunk\n\n") })
                    .chain(stream::pending()),
            )),
            Some(guard),
        );
        let (_parts, mut body) = response.into_parts();
        let frame = poll_fn(|cx| Pin::new(&mut body).poll_frame(cx))
            .await
            .expect("the response yields its initial chunk")
            .expect("the initial chunk is successful");
        assert_eq!(frame.into_data().unwrap(), "data: chunk\n\n");
        assert!(
            pool.clone().try_admit("codex", &acc, 1, false).is_none(),
            "slot should remain held while the response is pending"
        );

        drop(body); // client disconnect after reading one streamed chunk

        let reguard = tokio::time::timeout(std::time::Duration::from_secs(1), async {
            loop {
                if let Some(guard) = pool.clone().try_admit("codex", &acc, 1, false) {
                    break guard;
                }
                tokio::task::yield_now().await;
            }
        })
        .await
        .expect("account admission must be reusable after response drop");
        drop(reguard);
    }
}
