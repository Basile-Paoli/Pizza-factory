mod control;
mod heartbeat;
mod message;
mod receiver;
mod retry;
mod state;
mod sync;
mod version;

pub use control::Command;

use crate::gossip::control::start_command_listener;
use crate::gossip::receiver::start_listener;
use crate::gossip::state::{GossipState, LocalSkills, SharedGossipState};
use crate::gossip::sync::start_gossip_loop;
use std::net::{SocketAddr, UdpSocket};
use std::sync::{Arc, RwLock};
use std::sync::mpsc::Sender;

pub fn start_gossip(
    addr: SocketAddr,
    local_skills: LocalSkills,
    peer: Option<SocketAddr>,
) -> Result<Sender<Command>, Box<dyn std::error::Error>> {
    let shared_state: SharedGossipState = Arc::new(RwLock::new(GossipState::new(addr)));
    let local_skills = Arc::new(local_skills);

    let socket = UdpSocket::bind(addr)?;
    let (command_sender, command_receiver) = std::sync::mpsc::channel::<Command>();

    start_gossip_loop(shared_state.clone(), socket.try_clone()?, local_skills.clone());
    start_listener(socket.try_clone()?, shared_state.clone(), local_skills);
    start_command_listener(shared_state.clone(), command_receiver);

    if let Some(peer_addr) = peer {
        command_sender
            .send(Command::AddPeer { socket_addr: peer_addr })
            .expect("Failed to send AddPeer command");
    }

    Ok(command_sender)
}
