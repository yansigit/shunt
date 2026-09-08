use super::*;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[tokio::test]
async fn openai_chat_body_read_idle_is_bounded() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let upstream = tokio::spawn(async move {
        let (mut socket, _) = listener.accept().await.unwrap();
        let mut request = Vec::new();
        while !request.ends_with(b"\r\n\r\n") {
            assert!(request.len() < 4096, "fixture request headers too large");
            request.push(socket.read_u8().await.unwrap());
        }
        socket
            .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 100\r\n\r\n")
            .await
            .unwrap();
        std::future::pending::<()>().await;
    });
    let response = build_chat_client(Duration::from_millis(30))
        .get(format!("http://{address}"))
        .send()
        .await
        .unwrap();
    let result = tokio::time::timeout(Duration::from_secs(2), response.bytes()).await;
    upstream.abort();
    assert!(result
        .expect("body read must not wait indefinitely")
        .unwrap_err()
        .is_timeout());
}
