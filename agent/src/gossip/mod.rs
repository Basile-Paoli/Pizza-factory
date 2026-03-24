use crate::gossip::sync::start_gossip_loop;
use crate::gossip::state::{GossipState, LocalSkills, SharedGossipState};
use std::net::{SocketAddr, UdpSocket};
use std::sync::{Arc, RwLock};
use std::sync::mpsc::Sender;
use crate::gossip::control::{start_command_listener, Command};
use crate::gossip::receiver::start_listener;

mod sync;
mod receiver;
mod message;
mod heartbeat;
mod retry;
mod state;
mod version;
mod control;

pub fn start_gossip(
    addr: SocketAddr,
    local_skills: LocalSkills,
    peer: Option<SocketAddr>,
) -> Result<Sender<Command>, Box<dyn std::error::Error>> {
    let shared_state: SharedGossipState = Arc::new(RwLock::new(GossipState::new(addr)));

    let socket = UdpSocket::bind(addr)?;
    let (command_sender, command_receiver) = std::sync::mpsc::channel::<Command>();

    start_gossip_loop(
        shared_state.clone(),
        socket.try_clone()?,
        Arc::new(local_skills.clone()),
    );

    start_listener(socket.try_clone()?, shared_state.clone(), local_skills);

    start_command_listener(shared_state.clone(), command_receiver, &socket);

    if let Some(peer_addr) = peer {
        command_sender.send(Command::AddPeer { socket_addr: peer_addr }).expect("Failed to send AddPeer command");
    }

    Ok(command_sender)
}
