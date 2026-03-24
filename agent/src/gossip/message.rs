use crate::gossip::retry::retry_on_interval;
use crate::gossip::version::Version;
use serde::{Deserialize, Serialize};
use std::net::{SocketAddr, UdpSocket};

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
    SerializeError,
}

impl std::fmt::Display for GossipError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GossipError::IoError(e) => write!(f, "IO error: {e}"),
            GossipError::SerializeError => write!(f, "Serialization error"),
        }
    }
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
    ciborium::ser::into_writer(msg, &mut buf).map_err(|_| GossipError::SerializeError)?;

    retry_on_interval(
        || socket.send_to(&buf, address),
        std::time::Duration::from_secs(1),
        5,
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::UdpSocket;

    fn v1() -> Version {
        Version { counter: 1, generation: 0 }
    }

    fn bind() -> UdpSocket {
        UdpSocket::bind("127.0.0.1:0").unwrap()
    }

    fn recv_msg(socket: &UdpSocket) -> Message {
        socket.set_read_timeout(Some(std::time::Duration::from_secs(1))).unwrap();
        let mut buf = [0u8; 4096];
        let (len, _) = socket.recv_from(&mut buf).unwrap();
        ciborium::de::from_reader(&buf[..len]).unwrap()
    }

    #[test]
    fn ping_round_trip() {
        let sender = bind();
        let receiver = bind();
        let msg = Message::Ping(PingMessage { last_seen: 12345, version: v1() });
        send_message(&sender, receiver.local_addr().unwrap(), &msg).unwrap();
        match recv_msg(&receiver) {
            Message::Ping(p) => {
                assert_eq!(p.last_seen, 12345);
                assert_eq!(p.version, v1());
            }
            _ => panic!("Expected Ping"),
        }
    }

    #[test]
    fn pong_round_trip() {
        let sender = bind();
        let receiver = bind();
        let msg = Message::Pong(PongMessage { last_seen: 99, version: v1() });
        send_message(&sender, receiver.local_addr().unwrap(), &msg).unwrap();
        assert!(matches!(recv_msg(&receiver), Message::Pong(_)));
    }

    #[test]
    fn announce_round_trip() {
        let sender = bind();
        let receiver = bind();
        let node_addr: SocketAddr = "127.0.0.1:9999".parse().unwrap();
        let msg = Message::Announce(AnnounceMessage {
            node_addr,
            capabilities: vec!["cap1".into()],
            recipes: vec!["recipe1".into()],
            peers: vec![node_addr],
            version: v1(),
        });
        send_message(&sender, receiver.local_addr().unwrap(), &msg).unwrap();
        match recv_msg(&receiver) {
            Message::Announce(a) => {
                assert_eq!(a.node_addr, node_addr);
                assert_eq!(a.capabilities, vec!["cap1"]);
                assert_eq!(a.recipes, vec!["recipe1"]);
                assert_eq!(a.version, v1());
            }
            _ => panic!("Expected Announce"),
        }
    }
}
