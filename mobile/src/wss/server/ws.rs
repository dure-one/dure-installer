//! WebSocket handshake and message handling for the HTTPS/WSS server.

use crate::{dure_info, dure_debug, dure_warn, dure_error};
use async_tungstenite::{WebSocketStream, tungstenite::Message};
use base64::Engine;
use futures::io::AsyncWriteExt;
use futures::{FutureExt, SinkExt, StreamExt};
use std::io;
use std::time::{Duration, Instant};

use super::https::HttpRequest;
use super::messages as dure_messages;
use super::{ServerSettings, Stats, generate_session_id};

/// Calculate Sec-WebSocket-Accept header value
pub fn calculate_websocket_accept(key: &str) -> String {
    use sha1::{Digest, Sha1};

    const WEBSOCKET_GUID: &str = "258EAFA5-E914-47DA-95CA-C5AB0DC85B11";

    let mut hasher = Sha1::new();
    hasher.update(key.as_bytes());
    hasher.update(WEBSOCKET_GUID.as_bytes());
    let hash = hasher.finalize();

    base64::engine::general_purpose::STANDARD.encode(hash)
}

/// Perform WebSocket handshake (send 101 Switching Protocols response)
pub async fn perform_websocket_handshake<S: AsyncWriteExt + Unpin>(
    stream: &mut S,
    request: &HttpRequest,
    server_id: &str,
) -> io::Result<String> {
    let ws_key = request
        .headers
        .get("sec-websocket-key")
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Missing Sec-WebSocket-Key"))?;

    let accept_key = calculate_websocket_accept(ws_key);
    let session_id = generate_session_id();

    let mut response = String::from("HTTP/1.1 101 Switching Protocols\r\n");
    response.push_str("Upgrade: websocket\r\n");
    response.push_str("Connection: Upgrade\r\n");
    response.push_str(&format!("Sec-WebSocket-Accept: {}\r\n", accept_key));
    response.push_str(&format!("X-Session-ID: {}\r\n", session_id));
    response.push_str(&format!("X-Server-ID: {}\r\n", server_id));
    response.push_str(&format!(
        "Set-Cookie: session_id={}; Path=/; HttpOnly; Secure; SameSite=Strict\r\n",
        session_id
    ));
    response.push_str("\r\n");

    stream.write_all(response.as_bytes()).await?;
    stream.flush().await?;

    Ok(session_id)
}

pub async fn handle_websocket<S>(
    ws_stream: WebSocketStream<S>,
    peer_addr: std::net::SocketAddr,
    session_id: String,
    settings: ServerSettings,
    stats: Stats,
) -> io::Result<()>
where
    S: futures::io::AsyncRead + futures::io::AsyncWrite + Unpin + Send + 'static,
{
    use crate::storage::models::session;

    eprintln!(
        "WebSocket connection: {} (session: {})",
        peer_addr, session_id
    );

    if let Ok(mut db) = settings.db.lock() {
        let sess = session::Session::new(
            session_id.clone(),
            settings.domain.clone(),
            "wss".to_string(),
            peer_addr.to_string(),
        );
        if let Err(e) = session::store_session(&mut db, &sess) {
            dure_debug!("Failed to store session: {}", e);
        }
    }

    let (mut ws_sender, mut ws_receiver) = ws_stream.split();
    let mut last_activity = Instant::now();
    let mut last_ping = Instant::now();
    let timeout_duration = Duration::from_secs(settings.idle_timeout);

    loop {
        let time_since_activity = last_activity.elapsed();
        let remaining_timeout = if time_since_activity < timeout_duration {
            timeout_duration - time_since_activity
        } else {
            Duration::from_millis(1)
        };

        let mut next_msg = FutureExt::fuse(ws_receiver.next());
        let mut ping_timer = FutureExt::fuse(smol::Timer::after(Duration::from_secs(
            settings.ping_interval,
        )));
        let mut idle_timer = FutureExt::fuse(smol::Timer::after(remaining_timeout));

        futures::select! {
            msg = next_msg => {
                match msg {
                    Some(Ok(msg)) => {
                        last_activity = Instant::now();

                        if let Ok(mut db) = settings.db.lock() {
                            let _ = session::update_session_activity(&mut db, &session_id);
                        }

                        if msg.is_text() || msg.is_binary() {
                            stats.wss_message();

                            // Parse and route message
                            let response = if msg.is_text() {
                                let text = msg.to_text().unwrap();

                                match serde_json::from_str::<dure_messages::ClientMessage>(text) {
                                    Ok(client_msg) => {
                                        // Handle message and get response
                                        match super::handlers::handle_client_message(
                                            client_msg,
                                            &session_id,
                                            &settings,
                                        ).await {
                                            Ok(server_msg) => {
                                                // Serialize response
                                                match serde_json::to_string(&server_msg) {
                                                    Ok(json) => Message::Text(json.into()),
                                                    Err(e) => {
                                                        dure_debug!("[WS] Failed to serialize response: {}", e);
                                                        continue;
                                                    }
                                                }
                                            }
                                            Err(e) => {
                                                dure_debug!("[WS] Handler error: {}", e);
                                                // Send error response
                                                let error = dure_messages::ServerMessage::Error(
                                                    dure_messages::ErrorResponse {
                                                        code: "HANDLER_ERROR".to_string(),
                                                        message: e.to_string(),
                                                        request_id: None,
                                                        details: None,
                                                    }
                                                );
                                                Message::Text(serde_json::to_string(&error).unwrap().into())
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        dure_debug!("[WS] Failed to parse message: {}", e);
                                        // Send parse error
                                        let error = dure_messages::ServerMessage::Error(
                                            dure_messages::ErrorResponse {
                                                code: "PARSE_ERROR".to_string(),
                                                message: format!("Invalid message format: {}", e),
                                                request_id: None,
                                                details: None,
                                            }
                                        );
                                        Message::Text(serde_json::to_string(&error).unwrap().into())
                                    }
                                }
                            } else {
                                // Binary messages - echo for now
                                msg
                            };

                            if ws_sender.send(response).await.is_err() {
                                break;
                            }
                        } else if msg.is_pong() {
                            last_activity = Instant::now();
                        } else if msg.is_close() {
                            break;
                        }
                    }
                    Some(Err(e)) => {
                        dure_debug!("WebSocket error: {}", e);
                        break;
                    }
                    None => break,
                }
            }
            _ = ping_timer => {
                if last_ping.elapsed() >= Duration::from_secs(settings.ping_interval) {
                    if ws_sender.send(Message::Ping(Vec::new().into())).await.is_err() {
                        break;
                    }
                    last_ping = Instant::now();
                }
            }
            _ = idle_timer => {
                if last_activity.elapsed() >= Duration::from_secs(settings.idle_timeout) {
                    let _ = ws_sender.send(Message::Close(None)).await;
                    break;
                }
            }
        }
    }

    dure_debug!("WebSocket closed: {}", peer_addr);
    Ok(())
}
