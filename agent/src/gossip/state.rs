use crate::gossip::version::Version;
use std::net::SocketAddr;
use std::sync::{Arc, RwLock};
use std::time::{SystemTime, UNIX_EPOCH};

//TODO: share with other modules
//TODO: use real types instead of strings
#[derive(Clone)]
pub struct LocalSkills {
    pub capabilities: Vec<Capability>,
    pub recipes: Vec<Recipe>,
}

pub(super) type SharedGossipState = Arc<RwLock<GossipState>>;
pub(super) struct GossipState {
    pub(super) local_address: SocketAddr,
    pub(super) peers: Vec<Peer>,
    pub(super) known_peers: Vec<KnownPeer>,
    pub(super) version: Version,
}

impl GossipState {
    pub(super) fn new(local_address: SocketAddr) -> Self {
        Self {
            known_peers: Vec::new(),
            peers: Vec::new(),
            version: Version {
                counter: 1,
                generation: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .expect("Time went backwards")
                    .as_secs(),
            },
            local_address,
        }
    }
    pub(super) fn update_version(&mut self) {
        self.version.counter += 1;
    }

    pub(super) fn get_peer(&self, peer_addr: SocketAddr) -> Option<&Peer> {
        self.peers.iter().find(|p| p.address == peer_addr)
    }

    pub(super) fn get_peer_mut(&mut self, peer_addr: SocketAddr) -> Option<&mut Peer> {
        self.peers.iter_mut().find(|p| p.address == peer_addr)
    }

    pub(super) fn get_known_peer(&self, peer_addr: SocketAddr) -> Option<&KnownPeer> {
        self.known_peers.iter().find(|p| p.address == peer_addr)
    }

    pub(super) fn get_known_peer_mut(&mut self, peer_addr: SocketAddr) -> Option<&mut KnownPeer> {
        self.known_peers.iter_mut().find(|p| p.address == peer_addr)
    }
}

#[derive(Clone)]
pub(super) struct KnownPeer {
    pub(super) address: SocketAddr,
    pub(super) known_own_version: Option<Version>,
    pub(super) last_seen: u128,
}

#[derive(Clone)]
pub(super) struct Peer {
    pub(super) address: SocketAddr,
    pub(super) capabilities: Vec<Capability>,
    pub(super) recipes: Vec<Recipe>,
    pub(super) version: Version,
    pub(super) last_seen: u128,
}

//Temp types
type Capability = String;
type Recipe = String;

pub(super) fn remove_peer(state: &SharedGossipState, peer_addr: SocketAddr) {
    eprintln!("Removing peer: {peer_addr}");
    let mut state = state.write().expect("poisoned lock");
    state.known_peers.retain(|p| p.address != peer_addr);
    state.peers.retain(|p| p.address != peer_addr);
}
