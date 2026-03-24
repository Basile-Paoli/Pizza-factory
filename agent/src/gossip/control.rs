use crate::gossip::state::SharedGossipState;
use std::net::SocketAddr;

pub enum Command {
    AddPeer { socket_addr: SocketAddr },
}

pub(super) fn start_command_listener(
    shared_gossip_state: SharedGossipState,
    receiver: std::sync::mpsc::Receiver<Command>,
) {
    std::thread::spawn(move || {
        while let Ok(cmd) = receiver.recv() {
            match cmd {
                Command::AddPeer { socket_addr } => {
                    let mut state = shared_gossip_state.write().expect("poisoned lock");
                    state.add_known_peer(socket_addr);
                }
            }
        }
    });
}
