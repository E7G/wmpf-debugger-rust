use futures_util::{SinkExt, StreamExt};
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::broadcast;
use tokio_tungstenite::accept_async;
use tungstenite::Message as WsMessage;

use crate::cli::CliOptions;
use crate::logger::Logger;

pub async fn run_proxy_server(
    options: CliOptions,
    logger: Arc<Logger>,
    debug_tx: broadcast::Sender<String>,
    debug_rx: broadcast::Receiver<String>,
) {
    let addr = format!("127.0.0.1:{}", options.cdp_port);
    let listener = TcpListener::bind(&addr).await.unwrap();
    logger.info(&format!(
        "[server] proxy server running on ws://localhost:{}",
        options.cdp_port
    ));
    logger.info(&format!(
        "[server] link: devtools://devtools/bundled/inspector.html?ws=127.0.0.1:{}",
        options.cdp_port
    ));

    loop {
        let (stream, _) = match listener.accept().await {
            Ok(s) => s,
            Err(e) => {
                logger.error(&format!("[server] accept error: {}", e));
                continue;
            }
        };

        let logger = logger.clone();
        let debug_tx = debug_tx.clone();
        let mut debug_rx = debug_rx.resubscribe();

        tokio::spawn(async move {
            logger.info("[cdp] CDP client connected");

            let ws_stream = match accept_async(stream).await {
                Ok(s) => s,
                Err(e) => {
                    logger.error(&format!("[cdp] ws accept error: {}", e));
                    return;
                }
            };

            let (ws_tx, mut ws_rx) = ws_stream.split();

            // Spawn task to forward debug server messages to CDP client
            let ws_tx_fwd = Arc::new(tokio::sync::Mutex::new(ws_tx));
            let ws_tx_clone = ws_tx_fwd.clone();

            tokio::spawn(async move {
                while let Ok(msg) = debug_rx.recv().await {
                    let mut tx = ws_tx_clone.lock().await;
                    let _ = tx.send(WsMessage::Text(msg)).await;
                }
            });

            // Handle incoming messages from CDP client
            while let Some(msg) = ws_rx.next().await {
                match msg {
                    Ok(WsMessage::Text(text)) => {
                        let _ = debug_tx.send(text);
                    }
                    Ok(WsMessage::Close(_)) => {
                        logger.info("[cdp] CDP client disconnected");
                        break;
                    }
                    Err(e) => {
                        logger.error(&format!("[cdp] CDP client err: {}", e));
                        break;
                    }
                    _ => {}
                }
            }
        });
    }
}
