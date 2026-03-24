use crate::gossip::message::{AnnounceMessage, GossipError, Message, send_message};
use crate::gossip::state::{LocalSkills, SharedGossipState};
use std::net::{SocketAddr, UdpSocket};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

pub(super) fn start_gossip_loop(
    state: SharedGossipState,
    socket: UdpSocket,
    local_skills: Arc<LocalSkills>,
) {
    thread::spawn(move || loop {
        perform_gossip_round(state.clone(), &socket, local_skills.clone());
        thread::sleep(Duration::from_secs(1));
    });
}

pub(super) fn perform_gossip_round(
    state: SharedGossipState,
    socket: &UdpSocket,
    local_skills: Arc<LocalSkills>,
) {
    let peers_to_update = {
        let state = state.read().expect("poisoned lock");
        state.get_peers_to_update().collect::<Vec<_>>()
    };

    let handles = peers_to_update.iter().map(|&peer| {
        let socket_clone = socket.try_clone().expect("Failed to clone socket");
        let state = state.clone();
        let local_skills = local_skills.clone();
        thread::spawn(move || send_announce_message(&socket_clone, peer, &state, &local_skills))
    });

    for handle in handles {
        handle.join().expect("thread panicked").unwrap_or_else(|e| {
            eprintln!("Failed to send announce: {:?}", e);
        });
    }
}

pub(super) fn send_announce_message(
    socket: &UdpSocket,
    peer: SocketAddr,
    state: &SharedGossipState,
    local_skills: &LocalSkills,
) -> Result<(), GossipError> {
    let announce_message = {
        let state = state.read().expect("poisoned lock");
        Message::Announce(AnnounceMessage {
            // Use the configured local address, not socket.local_addr(), which may return
            // 0.0.0.0 if the socket was bound to the wildcard address.
            node_addr: state.local_address,
            capabilities: local_skills.capabilities.clone(),
            recipes: local_skills.recipes.clone(),
            peers: state.known_peers.iter().map(|p| p.address).collect(),
            version: state.version,
        })
    };

    send_message(socket, peer, &announce_message)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gossip::message::Message;
    use crate::gossip::state::{GossipState, KnownPeer};
    use std::net::UdpSocket;
    use std::sync::{Arc, RwLock};

    fn make_state(local_addr: SocketAddr) -> SharedGossipState {
        Arc::new(RwLock::new(GossipState::new(local_addr)))
    }

    fn skills() -> Arc<LocalSkills> {
        Arc::new(LocalSkills {
            capabilities: vec!["cap1".into()],
            recipes: vec!["recipe1".into()],
        })
    }

    fn recv_announce(socket: &UdpSocket) -> AnnounceMessage {
        socket.set_read_timeout(Some(std::time::Duration::from_secs(1))).unwrap();
        let mut buf = [0u8; 4096];
        let (len, _) = socket.recv_from(&mut buf).unwrap();
        match ciborium::de::from_reader(&buf[..len]).unwrap() {
            Message::Announce(a) => a,
            other => panic!("Expected Announce, got {other:?}"),
        }
    }

    fn add_known_peer_needing_update(state: &SharedGossipState, peer_addr: SocketAddr) {
        state.write().unwrap().known_peers.push(KnownPeer {
            address: peer_addr,
            known_own_version: None, // → needs Announce
            last_seen: 0,
        });
    }

    fn add_known_peer_up_to_date(state: &SharedGossipState, peer_addr: SocketAddr) {
        let mut s = state.write().unwrap();
        let version = s.version;
        s.known_peers.push(KnownPeer {
            address: peer_addr,
            known_own_version: Some(version), // → already up to date
            last_seen: 0,
        });
    }


    #[test]
    fn send_announce_carries_skills_and_version() {
        let sender = UdpSocket::bind("127.0.0.1:0").unwrap();
        let receiver = UdpSocket::bind("127.0.0.1:0").unwrap();
        let state = make_state("127.0.0.1:8882".parse().unwrap());

        send_announce_message(&sender, receiver.local_addr().unwrap(), &state, &skills()).unwrap();

        let a = recv_announce(&receiver);
        assert_eq!(a.capabilities, vec!["cap1"]);
        assert_eq!(a.recipes, vec!["recipe1"]);
        assert_eq!(a.version, state.read().unwrap().version);
    }

    #[test]
    fn send_announce_includes_known_peers_list() {
        let sender = UdpSocket::bind("127.0.0.1:0").unwrap();
        let receiver = UdpSocket::bind("127.0.0.1:0").unwrap();
        let third: SocketAddr = "127.0.0.1:9988".parse().unwrap();
        let state = make_state("127.0.0.1:8883".parse().unwrap());
        state.write().unwrap().add_known_peer(third);

        send_announce_message(&sender, receiver.local_addr().unwrap(), &state, &skills()).unwrap();

        let a = recv_announce(&receiver);
        assert!(a.peers.contains(&third));
    }

    #[test]
    fn gossip_round_sends_to_peer_needing_update() {
        let receiver = UdpSocket::bind("127.0.0.1:0").unwrap();
        let sender = UdpSocket::bind("127.0.0.1:0").unwrap();
        let state = make_state("127.0.0.1:8884".parse().unwrap());
        add_known_peer_needing_update(&state, receiver.local_addr().unwrap());

        perform_gossip_round(state, &sender, skills());

        recv_announce(&receiver); // panics on timeout if nothing received
    }

    #[test]
    fn gossip_round_skips_up_to_date_peer() {
        let receiver = UdpSocket::bind("127.0.0.1:0").unwrap();
        receiver.set_read_timeout(Some(std::time::Duration::from_millis(150))).unwrap();
        let sender = UdpSocket::bind("127.0.0.1:0").unwrap();
        let state = make_state("127.0.0.1:8885".parse().unwrap());
        add_known_peer_up_to_date(&state, receiver.local_addr().unwrap());

        perform_gossip_round(state, &sender, skills());

        let mut buf = [0u8; 1];
        assert!(receiver.recv_from(&mut buf).is_err(), "Should not receive any message");
    }
}
