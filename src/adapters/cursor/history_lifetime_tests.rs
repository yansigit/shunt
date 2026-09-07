use super::*;
use crate::adapters::cursor::connect::encode_connect_frame;

#[tokio::test]
async fn cursor_history_identity_guard_aborts_backpressured_sender() {
    let (tx, mut rx) = mpsc::channel::<u8>(1);
    tx.send(1).await.unwrap();
    let (started_tx, started_rx) = oneshot::channel();
    let sender = tokio::spawn(async move {
        started_tx.send(()).unwrap();
        let _ = tx.send(2).await;
    });
    let abort = sender.abort_handle();
    let (stop, _) = oneshot::channel();
    let guard = TurnGuard {
        _stop: Some(stop),
        _sender: Some(sender),
    };
    started_rx.await.unwrap();
    drop(guard);
    tokio::time::timeout(Duration::from_secs(1), async {
        while !abort.is_finished() {
            tokio::task::yield_now().await;
        }
    })
    .await
    .unwrap();
    assert_eq!(rx.recv().await, Some(1));
    assert_eq!(rx.recv().await, None);
}

#[tokio::test]
async fn cursor_continuation_guard_missing_or_closed_kv_sender_errors() {
    for closed in [false, true] {
        let (stop, _) = oneshot::channel();
        let (tx, rx) = mpsc::channel(1);
        drop(rx);
        let mut state = ReadState {
            bytes: futures_util::stream::empty().boxed(),
            decoder: ConnectFrameDecoder::new(),
            pending: VecDeque::new(),
            _guard: TurnGuard {
                _stop: Some(stop),
                _sender: None,
            },
            kv_store: RequestBlobStore::new(),
            kv_tx: closed.then_some(tx),
            got_text: false,
            finished: false,
        };
        let mut kv = field_varint(1, 1);
        kv.extend(field_ld(2, &field_ld(1, b"unknown")));
        state
            .ingest(&encode_connect_frame(field_ld(4, &kv), 0))
            .await;
        assert!(state.finished);
        assert!(state.pending.pop_front().unwrap().is_err());
        assert!(state.pending.is_empty());
    }
}
