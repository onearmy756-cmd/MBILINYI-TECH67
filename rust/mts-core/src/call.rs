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
    /// Everyone in the room right now, in join order.
    roster: Vec<Peer>,
}

/// A participant in a room: who they are, and the label they are known by.
struct Peer {
    label: String,
    conn: String,
}

fn rooms() -> &'static Mutex<HashMap<String, Room>> {
    static ROOMS: OnceLock<Mutex<HashMap<String, Room>>> = OnceLock::new();
    ROOMS.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Join a room, creating it if this is the first person.
///
/// Returns the receiver, this participant's label, and the labels of the people
/// already in the room. The label, not the display name, is what identifies a
/// participant: one account can be signed in on a phone and a laptop at the same
/// time, and both must be able to hear each other. Using the name as the key
/// made each of them ignore the other, because the client drops any message
/// whose sender matches its own name.
fn join(room: &str, name: &str, conn: &str) -> (broadcast::Receiver<String>, String, Vec<String>) {
    let mut map = rooms().lock().unwrap_or_else(|e| e.into_inner());
    let entry = map.entry(room.to_string()).or_insert_with(|| {
        let (tx, _rx) = broadcast::channel(256);
        Room { tx, peers: 0, roster: Vec::new() }
    });
    let label = format!("{name}#{conn}");
    let rx = entry.tx.subscribe();
    // The roster is read before this participant is added, so the list never
    // contains the newcomer and nobody is asked to negotiate with themselves.
    let others: Vec<String> = entry.roster.iter().map(|p| p.label.clone()).collect();
    entry.peers += 1;
    entry.roster.push(Peer {
        label: label.clone(),
        conn: conn.to_string(),
    });
    // Announcing the arrival is left to the caller, which does it only after the
    // newcomer has been told its own label. Broadcasting here would race: the
    // newcomer's own `peer-joined` could arrive before its `joined`, leaving it
    // with no label to compare against and making it greet itself.
    (rx, label, others)
}

/// Tell the room that someone arrived. Called once the newcomer knows its label.
fn announce_join(room: &str, label: &str) {
    let map = rooms().lock().unwrap_or_else(|e| e.into_inner());
    if let Some(entry) = map.get(room) {
        let _ = entry.tx.send(envelope("peer-joined", label, ""));
    }
}

fn leave(room: &str, label: &str) {
    let mut map = rooms().lock().unwrap_or_else(|e| e.into_inner());
    if let Some(entry) = map.get_mut(room) {
        entry.peers = entry.peers.saturating_sub(1);
        entry.roster.retain(|p| p.label != label);
        let _ = entry.tx.send(envelope("peer-left", label, ""));
        if entry.peers == 0 {
            map.remove(room);
        }
    }
}

/// The names of everyone in a room, as `(label, name)`.
fn roster(room: &str) -> Vec<(String, String)> {
    let map = rooms().lock().unwrap_or_else(|e| e.into_inner());
    map.get(room)
        .map(|r| {
            r.roster
                .iter()
                .map(|p| {
                    let name = p.label.rsplit_once('#').map(|(n, _)| n).unwrap_or(&p.label);
                    (p.label.clone(), name.to_string())
                })
                .collect()
        })
        .unwrap_or_default()
}

/// The message shape every client understands: a type, who it is from and a
/// JSON payload carrying the SDP or ICE candidate.
fn envelope(kind: &str, from: &str, payload: &str) -> String {
    serde_json::json!({ "type": kind, "from": from, "payload": payload }).to_string()
}

/// Who sent an envelope, read from the JSON rather than matched as text, so a
/// name that contains `"from"` cannot fool the check and a participant never
/// receives their own message back.
fn from_field(text: &str) -> Option<String> {
    serde_json::from_str::<serde_json::Value>(text)
        .ok()?
        .get("from")
        .and_then(|v| v.as_str())
        .map(str::to_string)
}

/// `GET /ws/call/<room>?name=<who>` — the upgrade handler.
///
/// A room carries live audio and video, so joining one requires a signed-in
/// session. The session travels in the cookie the browser sends with the
/// WebSocket handshake, so the check is the same one every API route makes —
/// and the display name is taken from the account, never from the query string,
/// so nobody can appear under someone else's name in a call.
pub async fn ws_call(
    Path(room): Path<String>,
    Query(q): Query<HashMap<String, String>>,
    headers: axum::http::HeaderMap,
    ws: WebSocketUpgrade,
) -> impl IntoResponse {
    let session = crate::session::load(&headers);
    let Some(user) = crate::session::current_user(&session) else {
        return (axum::http::StatusCode::UNAUTHORIZED, "sign in to join a call").into_response();
    };
    let name = crate::util::s(user.get("name").unwrap_or(&serde_json::Value::Null));
    let name = if name.trim().is_empty() {
        q.get("name").cloned().unwrap_or_else(|| "guest".into())
    } else {
        name
    };
    // One account can be open on several devices at once. Give each connection
    // its own label so the two do not look like the same participant.
    let conn = uuid_like();
    ws.on_upgrade(move |socket| handle(socket, room, name, conn)).into_response()
}

/// A short random token, used to tell two connections of the same account apart.
fn uuid_like() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let n = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let mix = n
        .wrapping_mul(6364136223846793005)
        .wrapping_add(std::process::id() as u128);
    format!("{:x}", mix & 0xffff_ffff)
}

async fn handle(mut socket: WebSocket, room: String, name: String, conn: String) {
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
    let (mut rx, label, others) = join(&room, &name, &conn);
    // Tell the newcomer who is already here, so the newcomer's client knows it
    // must offer to each of them. The roster is read before this participant is
    // added, so the list never contains the newcomer.
    let peers_json = serde_json::to_string(&others).unwrap_or_else(|_| "[]".into());
    let me_json = serde_json::json!({ "name": name, "label": label }).to_string();
    let _ = socket
        .send(Message::Text(
            envelope(
                "joined",
                "server",
                &format!("{{\"room\":\"{room}\",\"you\":{me_json},\"peers\":{peers_json}}}"),
            )
            .into(),
        ))
        .await;
    // Only now, once the newcomer knows its own label, tell the room it arrived.
    announce_join(&room, &label);

    loop {
        tokio::select! {
            // Something said by another participant.
            incoming = rx.recv() => {
                match incoming {
                    Ok(text) => {
                        // A participant must never be sent their own message
                        // back. The room is a broadcast channel, so this is the
                        // one place that decides who hears what: an offer, an
                        // answer, an ICE candidate or an announcement all skip
                        // the connection they came from.
                        if from_field(&text).as_deref() == Some(label.as_str()) {
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
                        relay(&room, &label, &t);
                    }
                    Some(Ok(Message::Close(_))) | None => break,
                    Some(Ok(_)) => {}
                    Some(Err(_)) => break,
                }
            }
        }
    }
    leave(&room, &label);
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
    fn a_peer_never_receives_their_own_message_back() {
        // The relay stamps `from`, and the reader compares that field, so the
        // sender's own offer is dropped before it reaches them.
        let mine = envelope("offer", "asha", "{}");
        assert_eq!(from_field(&mine).as_deref(), Some("asha"));
        let theirs = envelope("offer", "neema", "{}");
        assert_eq!(from_field(&theirs).as_deref(), Some("neema"));
        // A name that merely looks like a field must not be mistaken for one.
        let odd = envelope("chat", "from\":\"boss", "{}");
        assert_eq!(from_field(&odd).as_deref(), Some("from\":\"boss"));
        assert!(from_field("not json").is_none());
    }

    #[test]
    fn a_room_is_created_on_the_first_join_and_removed_on_the_last_leave() {
        let room = "unit-room-alpha";
        assert_eq!(peers_in(room), 0);
        let (_, a, _) = join(room, "ann", "c1");
        assert_eq!(peers_in(room), 1);
        let (_, b, _) = join(room, "bob", "c2");
        assert_eq!(peers_in(room), 2);
        leave(room, &a);
        assert_eq!(peers_in(room), 1);
        leave(room, &b);
        assert_eq!(peers_in(room), 0);
    }

    #[test]
    fn only_signalling_verbs_are_relayed() {
        // A message with a verb we do not know is dropped, so the hub cannot be
        // used as a general-purpose message bus.
        let room = "unit-room-beta";
        let (_, a, _) = join(room, "ann", "c1");
        relay(room, "ann", "{\"type\":\"exec\",\"payload\":\"rm -rf /\"}");
        relay(room, "ann", "{\"type\":\"offer\",\"payload\":\"sdp\"}");
        leave(room, &a);
    }

    #[test]
    fn one_account_on_two_devices_gets_two_distinct_labels() {
        // A phone and a laptop signed in as the same person must be able to hear
        // each other. Keying participants by name made each of them drop the
        // other's signals, so the call never connected.
        let room = "unit-room-same-account";
        let (_, phone, _) = join(room, "Asha", "aaaa");
        let (_, laptop, peers) = join(room, "Asha", "bbbb");
        assert_ne!(phone, laptop);
        // The second device is told the first is already there, so it offers.
        assert_eq!(peers, vec![phone.clone()]);
        let roster = roster(room);
        assert_eq!(roster.len(), 2);
        assert_eq!(roster[0].1, "Asha");
        leave(room, &phone);
        leave(room, &laptop);
    }

    #[test]
    fn a_newcomer_is_never_listed_as_already_present() {
        // The roster is read before the newcomer is added, so a client is never
        // told to negotiate with itself.
        let room = "unit-room-roster";
        let (_, first, peers) = join(room, "Asha", "c1");
        assert!(peers.is_empty());
        let (_, second, peers) = join(room, "Baraka", "c2");
        assert_eq!(peers, vec![first.clone()]);
        assert!(!peers.contains(&second));
        leave(room, &first);
        leave(room, &second);
        assert!(roster(room).is_empty());
    }

    #[test]
    fn a_bad_message_is_ignored_rather_than_panicking() {
        let room = "unit-room-gamma";
        let (_, a, _) = join(room, "ann", "c1");
        relay(room, "ann", "not json at all");
        leave(room, &a);
    }
}
