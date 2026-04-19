use crate::gossip::state::SharedGossipState;
use std::net::SocketAddr;

pub enum Command {
    AddPeer { socket_addr: SocketAddr },
    AddCapability { capability: String },
    AddRecipe { recipe: String },
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

                Command::AddCapability { capability } => {
                    let mut state = shared_gossip_state.write().expect("poisoned lock");
                    if !state.local_skills.capabilities.contains(&capability) {
                        state.local_skills.capabilities.push(capability);
                        state.update_version();
                    }
                }

                Command::AddRecipe { recipe } => {
                    let mut state = shared_gossip_state.write().expect("poisoned lock");
                    if !state.local_skills.recipes.contains(&recipe) {
                        state.local_skills.recipes.push(recipe);
                        state.update_version();
                    }
                }
            }
        }
    });
}
