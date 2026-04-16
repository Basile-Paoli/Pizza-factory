mod control;
mod heartbeat;
mod message;
mod receiver;
mod retry;
mod state;
mod sync;
mod version;

pub use control::Command;
pub use state::LocalSkills;

use crate::gossip::control::start_command_listener;
use crate::gossip::receiver::start_listener;
use crate::gossip::state::{GossipState, SharedGossipState};
use crate::gossip::sync::start_gossip_loop;
use std::collections::HashSet;
use std::net::{SocketAddr, UdpSocket};
use std::sync::{Arc, RwLock};
use std::sync::mpsc::Sender;

/// Handle vers l'état gossip, utilisé par le serveur TCP pour interroger les pairs.
///
/// Permet de savoir quels agents sont disponibles et ce qu'ils savent faire,
/// sans exposer les détails internes de l'état gossip.
pub struct GossipHandle {
    state: SharedGossipState,
}

impl GossipHandle {

    pub fn find_peer_for_action(&self, action: &str) -> Option<SocketAddr> {
        self.state
            .read()
            .expect("lock gossip empoisonné")
            .find_peer_for_action(action)
    }


    pub fn get_all_peer_capabilities(&self) -> HashSet<String> {
        self.state
            .read()
            .expect("lock gossip empoisonné")
            .get_all_peer_capabilities()
    }


    pub fn get_all_peer_recipe_names(&self) -> Vec<String> {
        self.state
            .read()
            .expect("lock gossip empoisonné")
            .get_all_peer_recipe_names()
    }
}

/// Démarre le service gossip UDP.
///
/// Retourne :
/// - Un [`Sender<Command>`] pour envoyer des commandes au gossip (ex: ajouter un pair)
/// - Un [`GossipHandle`] pour interroger l'état du réseau depuis le serveur TCP
pub fn start_gossip(
    addr: SocketAddr,
    local_skills: LocalSkills,
    peer: Option<SocketAddr>,
) -> Result<(Sender<Command>, GossipHandle), Box<dyn std::error::Error>> {
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
            .expect("Impossible d'envoyer AddPeer");
    }

    let handle = GossipHandle { state: shared_state };
    Ok((command_sender, handle))
}
