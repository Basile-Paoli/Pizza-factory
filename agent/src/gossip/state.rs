use crate::gossip::version::Version;
use std::net::SocketAddr;
use std::sync::{Arc, RwLock};
use std::time::{self, SystemTime, UNIX_EPOCH};

//TODO: use real types instead of strings
type Capability = String;
type Recipe = String;

#[derive(Clone)]
pub struct LocalSkills {
    pub capabilities: Vec<String>,
    pub recipes: Vec<String>,
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

    pub(super) fn remove_peer(&mut self, peer_addr: SocketAddr) {
        eprintln!("Removing peer: {peer_addr}");
        self.known_peers.retain(|p| p.address != peer_addr);
        self.peers.retain(|p| p.address != peer_addr);
    }

    /// Returns addresses of known peers whose version differs from ours (i.e. they need an Announce).
    pub(super) fn get_peers_to_update(&self) -> impl Iterator<Item = SocketAddr> + '_ {
        self.known_peers.iter().filter_map(|p| {
            if p.known_own_version != Some(self.version) {
                Some(p.address)
            } else {
                None
            }
        })
    }

    pub(super) fn add_known_peer(&mut self, peer_addr: SocketAddr) {
        if self.get_known_peer(peer_addr).is_none() {
            self.known_peers.push(KnownPeer {
                address: peer_addr,
                known_own_version: None,
                last_seen: time::SystemTime::now()
                    .duration_since(time::UNIX_EPOCH)
                    .unwrap()
                    .as_millis(),
            });
        }
    }

    pub(super) fn find_peer_for_action(&self, action: &str) -> Option<SocketAddr> {
        self.peers
            .iter()
            .find(|p| p.capabilities.iter().any(|c| c == action))
            .map(|p| p.address)
    }

    pub(super) fn get_all_peer_capabilities(&self) -> std::collections::HashSet<String> {
        self.peers
            .iter()
            .flat_map(|p| p.capabilities.iter().cloned())
            .collect()
    }

    pub(super) fn get_all_peer_recipe_names(&self) -> Vec<String> {
        self.peers
            .iter()
            .flat_map(|p| p.recipes.iter().cloned())
            .collect()
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
