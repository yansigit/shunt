use super::*;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
    sync::Notify,
    time::{timeout, Duration},
};

const DEADLINE: Duration = Duration::from_secs(3);

async fn read_request(socket: &mut tokio::net::TcpStream) {
    let mut bytes = Vec::new();
    loop {
        let mut buf = [0; 2048];
        let read = timeout(DEADLINE, socket.read(&mut buf))
            .await
            .unwrap()
            .unwrap();
        assert!(read > 0);
        bytes.extend_from_slice(&buf[..read]);
        assert!(bytes.len() <= 65536);
        if let Some(end) = bytes.windows(4).position(|v| v == b"\r\n\r\n") {
            let head = std::str::from_utf8(&bytes[..end])
                .unwrap()
                .to_ascii_lowercase();
            let length = head
                .lines()
                .find_map(|l| l.strip_prefix("content-length:"))
                .unwrap()
                .trim()
                .parse::<usize>()
                .unwrap();
            if bytes.len() >= end + 4 + length {
                return;
            }
        }
    }
}

// AI-SPEC 11: a real TCP downstream disconnect, not just dropping an in-process
// response future. The upstream's first socket must close and capacity must
// admit exactly one subsequent request. All server tasks are joined or aborted.
async fn cancellation_case(stage: &'static str, streaming: bool) {
    assert!(can_bind_loopback());
    let _lock = lock_openai_chat_env().await;
    let _key = EnvVarGuard::set("SHUNT_OPENAI_CHAT_CONFORMANCE_KEY", "fixture-openai-key");
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let ready = Arc::new(Notify::new());
    let closed = Arc::new(Notify::new());
    let hits = Arc::new(AtomicUsize::new(0));
    let (sent, dropped, requests) = (ready.clone(), closed.clone(), hits.clone());
    let backend = tokio::spawn(async move {
        let (mut first, _) = timeout(DEADLINE, listener.accept()).await.unwrap().unwrap();
        read_request(&mut first).await;
        requests.fetch_add(1, Ordering::SeqCst);
        if stage != "headers" {
            let (content_type, body) = if stage == "unary" {
                ("application/json", "{\"choices\":[".to_string())
            } else if stage == "tools" {
                (
                    "text/event-stream",
                    format!(
                        "data: {}\n\n",
                        json!({"choices":[{"delta":{"tool_calls":[{"index":0,"id":"a","function":{"name":"f","arguments":"{"}}]}}]})
                    ),
                )
            } else {
                (
                    "text/event-stream",
                    format!(
                        "data: {}\n\n",
                        json!({"choices":[{"delta":{"content":"early"}}]})
                    ),
                )
            };
            let wire = format!("HTTP/1.1 200 OK\r\ncontent-type: {content_type}\r\ntransfer-encoding: chunked\r\n\r\n{:x}\r\n{body}\r\n",body.len());
            first.write_all(wire.as_bytes()).await.unwrap();
            first.flush().await.unwrap();
        }
        sent.notify_one();
        let mut byte = [0];
        assert_eq!(
            timeout(DEADLINE, first.read(&mut byte))
                .await
                .expect("gateway must close upstream after downstream disconnect")
                .unwrap(),
            0
        );
        dropped.notify_one();
        let (mut second, _) = timeout(DEADLINE, listener.accept()).await.unwrap().unwrap();
        read_request(&mut second).await;
        requests.fetch_add(1, Ordering::SeqCst);
        let body = chat_completion_upstream().to_string();
        second.write_all(format!("HTTP/1.1 200 OK\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{body}",body.len()).as_bytes()).await.unwrap();
    });
    let mut backend = AbortOnDrop(backend);
    let mut config = single_provider_config(&format!("http://{addr}"));
    config.server.max_concurrent_requests = 1;
    let gateway = start_gateway(config).await;
    let mut client = tokio::net::TcpStream::connect(gateway.base_url.trim_start_matches("http://"))
        .await
        .unwrap();
    let body = if streaming {
        anthropic_streaming_request("claude-via-chat")
    } else {
        anthropic_request("claude-via-chat")
    };
    client.write_all(format!("POST /v1/messages HTTP/1.1\r\nHost: localhost\r\ncontent-type: application/json\r\ncontent-length: {}\r\n\r\n{body}",body.len()).as_bytes()).await.unwrap();
    timeout(DEADLINE, ready.notified())
        .await
        .expect("upstream received request and sent stage bytes");
    let saturated = timeout(
        DEADLINE,
        reqwest::Client::new()
            .post(format!("{}/v1/messages", gateway.base_url))
            .body(anthropic_request("claude-via-chat"))
            .send(),
    )
    .await
    .unwrap()
    .unwrap();
    assert_eq!(saturated.status(), StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(hits.load(Ordering::SeqCst), 1);
    drop(client);
    timeout(DEADLINE, closed.notified())
        .await
        .expect("upstream closure observed");
    let next = timeout(
        DEADLINE,
        reqwest::Client::new()
            .post(format!("{}/v1/messages", gateway.base_url))
            .body(anthropic_request("claude-via-chat"))
            .send(),
    )
    .await
    .expect("admission capacity recovered")
    .unwrap();
    assert_eq!(next.status(), StatusCode::OK);
    assert_eq!(
        next.json::<Value>().await.unwrap()["content"][0]["text"],
        "fixture reply"
    );
    assert_eq!(hits.load(Ordering::SeqCst), 2);
    timeout(DEADLINE, &mut backend.0)
        .await
        .expect("upstream server task completed")
        .unwrap();
}

struct AbortOnDrop(tokio::task::JoinHandle<()>);
impl Drop for AbortOnDrop {
    fn drop(&mut self) {
        self.0.abort();
    }
}

#[tokio::test]
async fn openai_chat_cancel_before_headers_unary() {
    cancellation_case("headers", false).await;
}
#[tokio::test]
async fn openai_chat_cancel_before_headers_streaming() {
    cancellation_case("headers", true).await;
}
#[tokio::test]
async fn openai_chat_cancel_unary_body() {
    cancellation_case("unary", false).await;
}
#[tokio::test]
async fn openai_chat_cancel_stream_text() {
    cancellation_case("text", true).await;
}
#[tokio::test]
async fn openai_chat_cancel_stream_tool_arguments() {
    cancellation_case("tools", true).await;
}
