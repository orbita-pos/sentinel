use anyhow::{Context, Result};
use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::tungstenite::Message;

use super::tls;
use super::types::{GameFlowPhase, LcuEvent, LockfileData};

/// Connect to the LCU WebSocket and process WAMP 1.0 events
pub async fn run_websocket(
    lockfile: &LockfileData,
    event_tx: tokio::sync::broadcast::Sender<LcuEvent>,
) -> Result<()> {
    let url = format!("wss://127.0.0.1:{}", lockfile.port);

    // Build auth header
    let encoded = base64::Engine::encode(
        &base64::engine::general_purpose::STANDARD,
        format!("riot:{}", lockfile.password),
    );
    let auth_header = format!("Basic {encoded}");

    let request = tokio_tungstenite::tungstenite::http::Request::builder()
        .uri(&url)
        .header("Authorization", &auth_header)
        .header("Sec-WebSocket-Key", tokio_tungstenite::tungstenite::handshake::client::generate_key())
        .header("Sec-WebSocket-Version", "13")
        .header("Connection", "Upgrade")
        .header("Upgrade", "websocket")
        .header("Host", format!("127.0.0.1:{}", lockfile.port))
        .body(())
        .context("Failed to build WebSocket request")?;

    let connector = tls::riot_tls_connector();

    let (mut ws, _response) =
        tokio_tungstenite::connect_async_tls_with_config(request, None, false, Some(connector))
            .await
            .context("WebSocket connection failed")?;

    tracing::info!("LCU WebSocket connected on port {}", lockfile.port);

    // WAMP 1.0: Subscribe to all events
    // Message type 5 = SUBSCRIBE
    let subscribe_msg = serde_json::json!([5, "OnJsonApiEvent"]);
    ws.send(Message::Text(subscribe_msg.to_string().into()))
        .await
        .context("Failed to send WAMP subscribe")?;

    tracing::debug!("Subscribed to OnJsonApiEvent");

    // Process incoming messages
    while let Some(msg) = ws.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                if let Err(e) = process_wamp_message(&text, &event_tx) {
                    tracing::warn!("Failed to process WAMP message: {e}");
                }
            }
            Ok(Message::Close(_)) => {
                tracing::info!("LCU WebSocket closed by server");
                break;
            }
            Ok(_) => {} // Ignore ping/pong/binary
            Err(e) => {
                tracing::warn!("WebSocket error: {e}");
                break;
            }
        }
    }

    Ok(())
}

/// Parse a WAMP 1.0 event message and dispatch typed events
///
/// WAMP event format: [8, "TopicURI", EventPayload]
/// Where EventPayload is: {"uri": "/path", "eventType": "Create|Update|Delete", "data": ...}
fn process_wamp_message(
    text: &str,
    event_tx: &tokio::sync::broadcast::Sender<LcuEvent>,
) -> Result<()> {
    let msg: serde_json::Value = serde_json::from_str(text)?;

    let arr = msg.as_array().context("WAMP message is not an array")?;

    // Only process EVENT messages (type 8)
    let msg_type = arr
        .first()
        .and_then(|v| v.as_i64())
        .unwrap_or(0);

    if msg_type != 8 {
        return Ok(());
    }

    // arr[2] is the event payload
    let payload = arr.get(2).context("Missing WAMP payload")?;
    let uri = payload
        .get("uri")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let data = payload.get("data").cloned().unwrap_or(serde_json::Value::Null);

    route_event(uri, data, event_tx);
    Ok(())
}

/// Route an LCU event by its URI to the appropriate typed event
fn route_event(
    uri: &str,
    data: serde_json::Value,
    event_tx: &tokio::sync::broadcast::Sender<LcuEvent>,
) {
    match uri {
        "/lol-gameflow/v1/gameflow-phase" => {
            let phase_str = data.as_str().unwrap_or("Unknown");
            let phase = GameFlowPhase::from_str_lossy(phase_str);
            tracing::info!("Game flow changed: {phase}");
            let _ = event_tx.send(LcuEvent::GameFlowChanged { phase });
        }
        "/lol-champ-select/v1/session" => {
            tracing::debug!("Champ select update");
            let _ = event_tx.send(LcuEvent::ChampSelectUpdate { data });
        }
        "/lol-end-of-game/v1/eog-stats-block" => {
            tracing::info!("End of game stats received");
            let _ = event_tx.send(LcuEvent::EndOfGame { data });
        }
        _ => {
            // Log interesting URIs at trace level for debugging
            if uri.contains("gameflow") || uri.contains("champ-select") || uri.contains("end-of-game") {
                tracing::trace!("LCU event: {uri}");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_wamp_gameflow() {
        let (tx, mut rx) = tokio::sync::broadcast::channel(16);
        let msg = r#"[8, "OnJsonApiEvent", {"uri": "/lol-gameflow/v1/gameflow-phase", "eventType": "Update", "data": "ChampSelect"}]"#;

        process_wamp_message(msg, &tx).unwrap();

        let event = rx.try_recv().unwrap();
        match event {
            LcuEvent::GameFlowChanged { phase } => {
                assert_eq!(phase, GameFlowPhase::ChampSelect);
            }
            _ => panic!("Expected GameFlowChanged"),
        }
    }

    #[test]
    fn test_process_wamp_non_event() {
        let (tx, mut rx) = tokio::sync::broadcast::channel(16);
        // Type 3 = CALLRESULT, should be ignored
        let msg = r#"[3, "callId", {}]"#;
        process_wamp_message(msg, &tx).unwrap();
        assert!(rx.try_recv().is_err());
    }
}
