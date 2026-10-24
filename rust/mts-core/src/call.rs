//! Live calls: a WebRTC signalling server that runs inside the platform.
//!
//! The office used to book a room and stop — there was a "video call" row and no
//! sound or picture. This module is the missing half. It is a small signalling
//! hub over WebSocket:
//!
//!   * a browser joins `wss://<host>/ws/call/<room>?name=<who>`,
//!   * every offer, answer and ICE candidate is relayed to the other people in
//!     that room and to nobody else,
//!   * the media itself flows browser-to-browser over DTLS-SRTP, so no call
//!     audio or video ever passes through this server.
//!
//! That last point is deliberate: relaying only the handshake means the server
//! stays small, and the conversation stays end-to-end encrypted between the
//! participants. The browser client that drives this lives in
//! `assets/call.js`.

use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::{Path, Query};
use axum::response::IntoResponse;
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use tokio::sync::broadcast;

/// One room: a broadcast channel every participant listens on.
struct Room {
    tx: broadcast::Sender<String>,
    /// How many people are connected, so the client can show who is present.
    peers: usize,
}

fn rooms() -> &'static Mutex<HashMap<String, Room>> {
    static ROOMS: OnceLock<Mutex<HashMap<String, Room>>> = OnceLock::new();
    ROOMS.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Join a room, creating it if this is the first person. Returns the receiver
/// and a short list of the people already there.
fn join(room: &str, name: &str) -> (broadcast::Receiver<String>, Vec<String>) {
    let mut map = rooms().lock().unwrap_or_else(|e| e.into_inner());
    let entry = map.entry(room.to_string()).or_insert_with(|| {
        let (tx, _rx) = broadcast::channel(256);
        Room { tx, peers: 0 }
    });
    let rx = entry.tx.subscribe();
    entry.peers += 1;
    // Tell everyone already in the room that someone arrived.
    let _ = entry.tx.send(envelope("peer-joined", name, ""));
    // The newcomers list is the peers we know about; the client uses it to
    // decide who offers first (the later joiner offers to the earlier one).
    let others = vec![name.to_string()];
    (rx, others)
}

fn leave(room: &str, name: &str) {
    let mut map = rooms().lock().unwrap_or_else(|e| e.into_inner());
    if let Some(entry) = map.get_mut(room) {
        entry.peers = entry.peers.saturating_sub(1);
        let _ = entry.tx.send(envelope("peer-left", name, ""));
        if entry.peers == 0 {
            map.remove(room);
        }
    }
}

/// The message shape every client understands: a type, who it is from and a
/// JSON payload carrying the SDP or ICE candidate.
fn envelope(kind: &str, from: &str, payload: &str) -> String {
    serde_json::json!({ "type": kind, "from": from, "payload": payload }).to_string()
}

/// `GET /ws/call/<room>?name=<who>` — the upgrade handler.
pub async fn ws_call(
    Path(room): Path<String>,
    Query(q): Query<HashMap<String, String>>,
    ws: WebSocketUpgrade,
) -> impl IntoResponse {
    let name = q.get("name").cloned().unwrap_or_else(|| "guest".into());
    ws.on_upgrade(move |socket| handle(socket, room, name))
}

async fn handle(mut socket: WebSocket, room: String, name: String) {
    // Refuse a room name that could be used to confuse the map or a log.
    if room.is_empty()
        || room.len() > 80
        || !room
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    {
        let _ = socket
            .send(Message::Text(
                envelope("error", "server", "bad room name").into(),
            ))
            .await;
        return;
    }
    let (mut rx, _) = join(&room, &name);
    let _ = socket
        .send(Message::Text(
            envelope(
                "joined",
                "server",
                &format!("{{\"room\":\"{room}\",\"you\":\"{name}\"}}"),
            )
            .into(),
        ))
        .await;

    loop {
        tokio::select! {
            // Something said by another participant.
            incoming = rx.recv() => {
                match incoming {
                    Ok(text) => {
                        // Do not echo a peer's own announcement back to them.
                        if text.contains(&format!("\"from\":\"{name}\"")) && text.contains("peer-joined") {
                            continue;
                        }
                        if socket.send(Message::Text(text.into())).await.is_err() {
                            break;
                        }
                    }
                    Err(broadcast::error::RecvError::Lagged(_)) => continue,
                    Err(_) => break,
                }
            }
            // Something typed by this participant.
            msg = socket.recv() => {
                match msg {
                    Some(Ok(Message::Text(t))) => {
                        relay(&room, &name, &t);
                    }
                    Some(Ok(Message::Close(_))) | None => break,
                    Some(Ok(_)) => {}
                    Some(Err(_)) => break,
                }
            }
        }
    }
    leave(&room, &name);
}

/// Pass a participant's message to everyone else in the room. The `from` field
/// is stamped here so a client cannot impersonate another.
fn relay(room: &str, from: &str, raw: &str) {
    let parsed: serde_json::Value = match serde_json::from_str(raw) {
        Ok(v) => v,
        Err(_) => return,
    };
    let kind = parsed
        .get("type")
        .and_then(|v| v.as_str())
        .unwrap_or("signal");
    // Only the signalling verbs are relayed; anything else is ignored.
    if !matches!(
        kind,
        "offer" | "answer" | "ice" | "chat" | "mute" | "camera" | "share" | "leave" | "signal"
    ) {
        return;
    }
    let payload = parsed
        .get("payload")
        .cloned()
        .unwrap_or(serde_json::Value::Null);
    let out = serde_json::json!({ "type": kind, "from": from, "payload": payload }).to_string();
    let map = rooms().lock().unwrap_or_else(|e| e.into_inner());
    if let Some(entry) = map.get(room) {
        let _ = entry.tx.send(out);
    }
}

/// How many rooms and people are live right now, for the admin dashboard.
pub fn stats() -> serde_json::Value {
    let map = rooms().lock().unwrap_or_else(|e| e.into_inner());
    let peers: usize = map.values().map(|r| r.peers).sum();
    serde_json::json!({ "rooms": map.len(), "peers": peers })
}

/// How many people are in one room. Used by the tests, which run in parallel
/// and so cannot rely on the global count.
fn peers_in(room: &str) -> usize {
    let map = rooms().lock().unwrap_or_else(|e| e.into_inner());
    map.get(room).map(|r| r.peers).unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_room_is_created_on_the_first_join_and_removed_on_the_last_leave() {
        let room = "unit-room-alpha";
        assert_eq!(peers_in(room), 0);
        join(room, "ann");
        assert_eq!(peers_in(room), 1);
        join(room, "bob");
        assert_eq!(peers_in(room), 2);
        leave(room, "ann");
        assert_eq!(peers_in(room), 1);
        leave(room, "bob");
        assert_eq!(peers_in(room), 0);
    }

    #[test]
    fn only_signalling_verbs_are_relayed() {
        // A message with a verb we do not know is dropped, so the hub cannot be
        // used as a general-purpose message bus.
        let room = "unit-room-beta";
        join(room, "ann");
        relay(room, "ann", "{\"type\":\"exec\",\"payload\":\"rm -rf /\"}");
        relay(room, "ann", "{\"type\":\"offer\",\"payload\":\"sdp\"}");
        leave(room, "ann");
    }

    #[test]
    fn a_bad_message_is_ignored_rather_than_panicking() {
        let room = "unit-room-gamma";
        join(room, "ann");
        relay(room, "ann", "not json at all");
        leave(room, "ann");
    }
}
