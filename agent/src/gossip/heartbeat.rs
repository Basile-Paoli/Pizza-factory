use crate::gossip::message::{GossipError, Message, PingMessage, PongMessage, send_message};
use crate::gossip::state::{Peer, SharedGossipState, remove_peer};
use std::net::{SocketAddr, UdpSocket};
use std::thread;
use std::time::{Duration, SystemTime};

const REFRESH_TIMEOUT: Duration = Duration::from_secs(10);
const REFRESH_DELAY: Duration = Duration::from_secs(1);
pub(super) fn ping_loop(
    address: SocketAddr,
    shared_gossip_state: SharedGossipState,
    udp_socket: &UdpSocket,
) -> Result<(), GossipError> {
    loop {
        let (last_seen, version) = {
            let state = shared_gossip_state.read().expect("poisoned lock");
            let peer = state.peers.iter().find(|p| p.address == address);
            match peer {
                Some(p) => (p.last_seen, p.version),
                None => return Ok(()),
            }
        };

        let last_seen_time = SystemTime::UNIX_EPOCH + Duration::from_millis(last_seen as u64);
        if last_seen_time.elapsed().unwrap_or(Duration::ZERO) > REFRESH_TIMEOUT {
            remove_peer(&shared_gossip_state, address);
            return Ok(());
        }

        let msg = Message::Ping(PingMessage { last_seen, version });

        send_message(udp_socket, address, &msg)?;

        thread::sleep(REFRESH_DELAY);
    }
}

pub(super) fn send_pong(
    peer: &Peer,
    shared_gossip_state: &SharedGossipState,
    udp_socket: &UdpSocket,
) -> Result<(), GossipError> {
    let pong_msg = Message::Pong(PongMessage {
        last_seen: peer.last_seen,
        version: {
            let state = shared_gossip_state.read().expect("poisoned lock");
            state.version.clone()
        },
    });

    send_message(udp_socket, peer.address, &pong_msg)
}
