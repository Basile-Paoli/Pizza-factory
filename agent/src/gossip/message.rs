use crate::gossip::version::Version;
use serde::{Deserialize, Serialize};
use std::net::{SocketAddr, UdpSocket};
use crate::gossip::retry::retry_on_interval;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(super) enum Message {
    Ping(PingMessage),
    Pong(PongMessage),
    Announce(AnnounceMessage),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(super) struct PingMessage {
    pub last_seen: u128,
    pub version: Version,
}
pub(super) type PongMessage = PingMessage;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(super) struct AnnounceMessage {
    pub(super) node_addr: SocketAddr,
    pub(super) capabilities: Vec<String>,
    pub(super) recipes: Vec<String>,
    pub(super) peers: Vec<SocketAddr>,
    pub(super) version: Version,
}

#[derive(Debug)]
pub(super) enum GossipError {
    IoError(std::io::Error),
}

impl From<std::io::Error> for GossipError {
    fn from(e: std::io::Error) -> Self {
        GossipError::IoError(e)
    }
}

pub(super) fn send_message(
    socket: &UdpSocket,
    address: SocketAddr,
    msg: &Message,
) -> Result<(), GossipError> {
    let mut buf = Vec::new();
    ciborium::ser::into_writer(msg, &mut buf).expect("Failed to serialize message");

    retry_on_interval(
        || socket.send_to(&buf, address),
        std::time::Duration::from_secs(1),
        5,
    )?;
    Ok(())
}
