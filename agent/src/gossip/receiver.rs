use crate::gossip::message::{AnnounceMessage, GossipError, Message};
use crate::gossip::state::{KnownPeer, LocalSkills, Peer, SharedGossipState};
use crate::gossip::version::Version;
use std::net::{SocketAddr, UdpSocket};
use std::{thread, time};
use crate::gossip::heartbeat::{ping_loop, send_pong};
use crate::gossip::sync::send_announce_message;

pub(super) fn start_listener(
    socket: UdpSocket,
    shared_gossip_state: SharedGossipState,
    local_skills: LocalSkills,
) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        let mut buf = [0; 1 << 16];
        loop {
            match socket.recv_from(&mut buf) {
                Ok((len, src)) => {
                    let message: Result<Message, _> = ciborium::de::from_reader(&buf[..len]);
                    match message {
                        Err(e) => eprintln!("Error deserializing message from {src}: {e}"),
                        Ok(msg) => {
                            handle_message(msg, src, &shared_gossip_state, &local_skills, &socket).unwrap_or_else(|e| {
                                eprintln!("Error handling message from {src}: {e:?}");
                            });
                        }
                    }
                }
                Err(e) => eprintln!("Error receiving message: {e}"),
            }
        }
    })
}

fn handle_message(
    msg: Message,
    src: std::net::SocketAddr,
    shared_gossip_state: &SharedGossipState,
    local_skills: &LocalSkills,
    socket: &UdpSocket,
) -> Result<(), GossipError> {
    match msg {
        Message::Announce(announce) => {
            handle_announce(announce, src, shared_gossip_state, local_skills, socket)
        }
        Message::Ping(ping) => {
            let peer = if let Some(peer) = shared_gossip_state
                .read()
                .expect("poisoned lock")
                .get_peer(src)
            {
                peer.clone()
            } else {
                eprintln!("Received Ping from unknown peer {src}, ignoring");
                return Ok(());
            };

            update_peer_known_own_version(src, ping.version, shared_gossip_state);
            update_last_seen(src, shared_gossip_state);
            send_pong(&peer,  shared_gossip_state, socket)?;
            Ok(())
        }
        Message::Pong(pong) => {
            check_peer_version_change(src, pong.version, shared_gossip_state, local_skills, socket)?;
            update_last_seen(src, shared_gossip_state);
            Ok(())
        }
    }
}

fn handle_announce(
    announce: AnnounceMessage,
    src: SocketAddr,
    shared_gossip_state: &SharedGossipState,
    local_skills: &LocalSkills,
    socket: &UdpSocket,
) -> Result<(), GossipError> {
    if announce.node_addr != src {
        eprintln!(
            "Warning: Announce message from {src} contains node address {}, ignoring",
            announce.node_addr
        );
        return Ok(());
    }

    let (existing_peer, curr_version) = {
        let state = shared_gossip_state.read().expect("poisoned lock");
        (state.get_peer(src).cloned(), state.version.clone())
    };


    if let Some(peer) = existing_peer {
        if peer.version != announce.version {
                handle_peer_version_change(src, shared_gossip_state, &announce, socket)?;
        }
        let needs_update = {
            let state  = shared_gossip_state.read().expect("poisoned lock");
            let known_peer = state.get_known_peer(src);
            known_peer.map_or(true, |kp| kp.known_own_version != Some(curr_version))
        };
        if needs_update {
            send_announce_message(socket, src, shared_gossip_state, local_skills)?;
        }

        update_last_seen(src, shared_gossip_state);
        Ok(())
    } else {
        add_peer(src, announce.capabilities, announce.recipes, announce.version, shared_gossip_state, socket);
        Ok(())
    }
}

fn check_peer_version_change(
    peer_addr: SocketAddr,
    peer_version: Version,
    shared_gossip_state: &SharedGossipState,
    local_skills: &LocalSkills,
    socket: &UdpSocket,
) -> Result<(), GossipError> {
    {
        let state = shared_gossip_state.read().expect("poisoned lock");
        let existing_peer = if let Some(peer) = state.get_peer(peer_addr) {
            peer
        } else {
            return Ok(());
        };

        if existing_peer.version == peer_version {
            return Ok(());
        }
    }

    send_announce_message(socket, peer_addr, shared_gossip_state, local_skills)
}

fn update_peer_known_own_version(
    peer_addr: SocketAddr,
    own_version: Version,
    shared_gossip_state: &SharedGossipState,
) {
    let mut state = shared_gossip_state.write().expect("poisoned lock");
    if let Some(peer) = state.get_known_peer_mut(peer_addr) {
        peer.known_own_version = Some(own_version);
    }
}



fn add_peer(
    peer_addr: SocketAddr,
    capabilities: Vec<String>,
    recipes: Vec<String>,
    version: Version,
    shared_gossip_state: &SharedGossipState,
    udp_socket: &UdpSocket,
) {
    let now = time::SystemTime::now()
        .duration_since(time::UNIX_EPOCH)
        .unwrap()
        .as_millis();

    let is_new_peer = {
        let state = shared_gossip_state.read().expect("poisoned lock");
        state.get_peer(peer_addr).is_none()
    };
    if is_new_peer {
        add_known_peer(peer_addr, shared_gossip_state, &udp_socket);
    }

    let mut state = shared_gossip_state.write().expect("poisoned lock");
    if let Some(peer) = state.get_peer_mut(peer_addr) {
        peer.capabilities = capabilities;
        peer.recipes = recipes;
        peer.version = version;
        peer.last_seen = now;
    } else {
        state.peers.push(Peer {
            address: peer_addr,
            capabilities,
            recipes,
            version,
            last_seen: now,
        });
    }
    state.update_version();
}


fn update_last_seen(peer_addr: SocketAddr, shared_gossip_state: &SharedGossipState) {
    let mut state = shared_gossip_state.write().expect("poisoned lock");
    if let Some(peer) = state.get_peer_mut(peer_addr) {
        peer.last_seen = time::SystemTime::now()
            .duration_since(time::UNIX_EPOCH)
            .unwrap()
            .as_millis();
    }
}

fn handle_peer_version_change(
    peer_addr: SocketAddr,
    shared_gossip_state: &SharedGossipState,
    announce_message: &AnnounceMessage,
    udp_socket: &UdpSocket,
) -> Result<(), GossipError> {
    let new_peer_addrs: Vec<SocketAddr> = {
        let mut state = shared_gossip_state.write().expect("poisoned lock");
        if let Some(peer) = state.get_peer_mut(peer_addr) {
            peer.version = announce_message.version.clone();
            peer.capabilities = announce_message.capabilities.clone();
            peer.recipes = announce_message.recipes.clone();
        }

        let known_peers: Vec<_> = state.known_peers.iter().map(|p| p.address).collect();
        announce_message
            .peers
            .iter()
            .copied()
            .filter(|p| !known_peers.contains(p) && *p != state.local_address)
            .collect()
    };

    for new_peer in new_peer_addrs {
        add_known_peer(new_peer, shared_gossip_state, udp_socket);
    }
    Ok(())
}



pub(super) fn add_known_peer(peer_addr: SocketAddr, shared_gossip_state: &SharedGossipState, udp_socket: &UdpSocket) {
    let mut state = shared_gossip_state.write().expect("poisoned lock");
    if state.get_known_peer(peer_addr).is_none() {
        state.known_peers.push(KnownPeer {
            address: peer_addr,
            known_own_version: None,
            last_seen: time::SystemTime::now()
                .duration_since(time::UNIX_EPOCH)
                .unwrap()
                .as_millis(),
        });
    }

    let udp_socket = udp_socket.try_clone().expect("Failed to clone UDP socket for ping loop");
    let shared_gossip_state = shared_gossip_state.clone();

    thread::spawn(move || {
        ping_loop(peer_addr, shared_gossip_state, &udp_socket).unwrap_or_else(|e| {
            eprintln!("Error in ping loop for {peer_addr}: {e:?}");
        });
    });
}
