use crate::gossip::message::{AnnounceMessage, GossipError, Message, send_message};
use crate::gossip::state::{GossipState, LocalSkills, SharedGossipState};
use std::net::{SocketAddr, UdpSocket};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

impl GossipState {
    fn get_peers_to_update(&self) -> impl Iterator<Item = SocketAddr> {
        self.known_peers.iter().filter_map(|p| {
            if p.known_own_version != Some(self.version) {
                Some(p.address)
            } else {
                None
            }
        })
    }
}

pub(super) fn start_gossip_loop(
    state: SharedGossipState,
    socket: UdpSocket,
    local_skills: Arc<LocalSkills>,
) {
    thread::spawn(move || {
        loop {
            perform_gossip_round(state.clone(), &socket, local_skills.clone());
            thread::sleep(Duration::from_secs(1));
        }
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
            eprintln!("Failed to send message: {:?}", e);
        })
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
            node_addr: socket.local_addr().expect("Failed to get local address"),
            capabilities: local_skills.capabilities.clone(),
            recipes: local_skills.recipes.clone(),
            peers: state.known_peers.iter().map(|p| p.address).collect(),
            version: state.version.clone(),
        })
    };

    send_message(socket, peer, &announce_message)
}
