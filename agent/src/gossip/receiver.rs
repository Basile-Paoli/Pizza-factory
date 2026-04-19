use crate::gossip::heartbeat::{ping_loop, send_pong};
use crate::gossip::message::{AnnounceMessage, GossipError, Message, PingMessage, PongMessage};
use crate::gossip::state::{KnownPeer, LocalSkills, Peer, SharedGossipState};
use crate::gossip::sync::send_announce_message;
use crate::gossip::version::Version;
use std::net::{SocketAddr, UdpSocket};
use std::sync::Arc;
use std::{thread, time};

pub(super) fn start_listener(socket: UdpSocket, shared_gossip_state: SharedGossipState) {
    thread::spawn(move || {
        let mut buf = [0; 1 << 16];
        loop {
            match socket.recv_from(&mut buf) {
                Ok((len, src)) => {
                    let message: Result<Message, _> = ciborium::de::from_reader(&buf[..len]);
                    match message {
                        Err(e) => eprintln!("Error deserializing message from {src}: {e}"),
                        Ok(msg) => {
                            handle_message(msg, src, &shared_gossip_state, &socket).unwrap_or_else(
                                |e| {
                                    eprintln!("Error handling message from {src}: {e:?}");
                                },
                            );
                        }
                    }
                }
                Err(e) => eprintln!("Error receiving message: {e}"),
            }
        }
    });
}

fn handle_message(
    msg: Message,
    src: SocketAddr,
    shared_gossip_state: &SharedGossipState,
    socket: &UdpSocket,
) -> Result<(), GossipError> {
    match msg {
        Message::Announce(announce) => handle_announce(announce, src, shared_gossip_state, socket),
        Message::Ping(ping) => handle_ping(ping, src, socket, shared_gossip_state),
        Message::Pong(pong) => handle_pong(pong, src, shared_gossip_state, socket)?,
    }
}

fn handle_pong(
    pong: PongMessage,
    src: SocketAddr,
    shared_gossip_state: &SharedGossipState,
    socket: &UdpSocket,
) -> Result<Result<(), GossipError>, GossipError> {
    let version_changed = shared_gossip_state
        .read()
        .expect("poisoned lock")
        .get_peer(src)
        .map_or(false, |p| p.version != pong.version);

    update_last_seen(src, shared_gossip_state);

    if version_changed {
        let socket_clone = socket.try_clone()?;
        let state_clone = shared_gossip_state.clone();
        thread::spawn(move || {
            send_announce_message(&socket_clone, src, &state_clone)
                .unwrap_or_else(|e| eprintln!("Failed to send announce to {src}: {e:?}"));
        });
    }
    Ok(Ok(()))
}

fn handle_ping(
    ping: PingMessage,
    src: SocketAddr,
    socket: &UdpSocket,
    shared_gossip_state: &SharedGossipState,
) -> Result<(), GossipError> {
    let peer = {
        let mut state = shared_gossip_state.write().expect("poisoned lock");
        match state.get_peer(src) {
            Some(peer) => peer.clone(),
            None => {
                state.known_peers.push(KnownPeer {
                    address: src,
                    known_own_version: None,
                    last_seen: 0,
                });
                return Ok(());
            }
        }
    };

    update_peer_known_own_version(src, ping.version, shared_gossip_state);
    update_last_seen(src, shared_gossip_state);

    let socket_clone = socket.try_clone()?;
    let state_clone = shared_gossip_state.clone();
    thread::spawn(move || {
        send_pong(&peer, &state_clone, &socket_clone).unwrap_or_else(|e| {
            eprintln!("Error sending pong to {}: {e:?}", peer.address);
        });
    });
    Ok(())
}

fn handle_announce(
    announce: AnnounceMessage,
    src: SocketAddr,
    shared_gossip_state: &SharedGossipState,
    socket: &UdpSocket,
) -> Result<(), GossipError> {
    if announce.node_addr.0 != src {
        eprintln!(
            "Warning: Announce from {src} contains node_addr {}, ignoring",
            announce.node_addr.0
        );
        return Ok(());
    }

    let (existing_peer, curr_version) = {
        let state = shared_gossip_state.read().expect("poisoned lock");
        (state.get_peer(src).cloned(), state.version)
    };

    if let Some(peer) = existing_peer {
        if peer.version != announce.version {
            handle_peer_version_change(src, shared_gossip_state, &announce);
        }

        let needs_update = shared_gossip_state
            .read()
            .expect("poisoned lock")
            .get_known_peer(src)
            .map_or(true, |kp| kp.known_own_version != Some(curr_version));

        if needs_update {
            let socket_clone = socket.try_clone()?;
            let state_clone = shared_gossip_state.clone();
            thread::spawn(move || {
                send_announce_message(&socket_clone, src, &state_clone)
                    .unwrap_or_else(|e| eprintln!("Failed to send announce to {src}: {e:?}"));
            });
        }

        update_last_seen(src, shared_gossip_state);
        Ok(())
    } else {
        add_peer(
            src,
            announce.capabilities,
            announce.recipes,
            announce.version,
            shared_gossip_state,
            socket,
        );
        // Discover third-party peers advertised in the Announce even for first contact.
        let new_peers: Vec<SocketAddr> = {
            let state = shared_gossip_state.read().expect("poisoned lock");
            announce
                .peers
                .iter()
                .map(|p| p.0)
                .filter(|p| state.get_known_peer(*p).is_none() && *p != state.local_address)
                .collect()
        };
        for p in new_peers {
            shared_gossip_state
                .write()
                .expect("poisoned lock")
                .add_known_peer(p);
        }
        Ok(())
    }
}

fn handle_peer_version_change(
    peer_addr: SocketAddr,
    shared_gossip_state: &SharedGossipState,
    announce_message: &AnnounceMessage,
) {
    let new_peer_addrs: Vec<SocketAddr> = {
        let mut state = shared_gossip_state.write().expect("poisoned lock");
        if let Some(peer) = state.get_peer_mut(peer_addr) {
            peer.version = announce_message.version;
            peer.capabilities = announce_message.capabilities.clone();
            peer.recipes = announce_message.recipes.clone();
        }

        let known: Vec<_> = state.known_peers.iter().map(|p| p.address).collect();
        announce_message
            .peers
            .iter()
            .map(|p| p.0)
            .filter(|p| !known.contains(p) && *p != state.local_address)
            .collect()
    };

    for new_peer in new_peer_addrs {
        let mut state = shared_gossip_state.write().expect("poisoned lock");
        state.add_known_peer(new_peer);
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
        let mut state = shared_gossip_state.write().expect("poisoned lock");

        if state.get_known_peer(peer_addr).is_none() {
            state.known_peers.push(KnownPeer {
                address: peer_addr,
                known_own_version: None,
                last_seen: now,
            });
        }

        let is_new = state.get_peer(peer_addr).is_none();
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
        is_new
    };

    if is_new_peer {
        let socket_clone = udp_socket
            .try_clone()
            .expect("Failed to clone UDP socket for ping loop");
        let state_clone = shared_gossip_state.clone();
        thread::spawn(move || {
            ping_loop(peer_addr, state_clone, &socket_clone, Default::default()).unwrap_or_else(
                |e| {
                    eprintln!("Error in ping loop for {peer_addr}: {e:?}");
                },
            );
        });
    }
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

fn update_last_seen(peer_addr: SocketAddr, shared_gossip_state: &SharedGossipState) {
    let now = time::SystemTime::now()
        .duration_since(time::UNIX_EPOCH)
        .unwrap()
        .as_millis();
    let mut state = shared_gossip_state.write().expect("poisoned lock");
    if let Some(peer) = state.get_peer_mut(peer_addr) {
        peer.last_seen = now;
    }
    if let Some(known) = state.get_known_peer_mut(peer_addr) {
        known.last_seen = now;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gossip::message::{AnnounceMessage, Message};
    use crate::gossip::state::GossipState;
    use crate::gossip::version::Version;
    use shared::TaggedSocketAddr;
    use std::net::UdpSocket;
    use std::sync::{Arc, RwLock};
    use std::time::{Duration, Instant};

    fn v(counter: u64) -> Version {
        Version {
            counter,
            generation: 0,
        }
    }

    fn make_state(local_addr: SocketAddr) -> SharedGossipState {
        Arc::new(RwLock::new(GossipState::new(
            local_addr,
            LocalSkills {
                capabilities: vec![],
                recipes: vec![],
            },
        )))
    }

    fn send_announce(socket: &UdpSocket, dest: SocketAddr, announce: AnnounceMessage) {
        let msg = Message::Announce(announce);
        let mut buf = Vec::new();
        ciborium::ser::into_writer(&msg, &mut buf).unwrap();
        socket.send_to(&buf, dest).unwrap();
    }

    fn wait_until(pred: impl Fn() -> bool) {
        let deadline = Instant::now() + Duration::from_secs(1);
        while !pred() {
            assert!(Instant::now() < deadline, "Timed out waiting for condition");
            std::thread::sleep(Duration::from_millis(10));
        }
    }

    #[test]
    fn listener_adds_peer_on_announce() {
        let listener_sock = UdpSocket::bind("127.0.0.1:0").unwrap();
        let listener_addr = listener_sock.local_addr().unwrap();
        let state = make_state(listener_addr);
        start_listener(listener_sock, state.clone());

        let sender = UdpSocket::bind("127.0.0.1:0").unwrap();
        let sender_addr = sender.local_addr().unwrap();
        send_announce(
            &sender,
            listener_addr,
            AnnounceMessage {
                node_addr: TaggedSocketAddr::new(sender_addr),
                capabilities: vec!["x".into()],
                recipes: vec!["y".into()],
                peers: vec![],
                version: v(1),
            },
        );

        wait_until(|| state.read().unwrap().get_peer(sender_addr).is_some());
        let s = state.read().unwrap();
        let peer = s.get_peer(sender_addr).unwrap();
        assert_eq!(peer.capabilities, vec!["x"]);
        assert_eq!(peer.recipes, vec!["y"]);
    }

    #[test]
    fn listener_ignores_spoofed_node_addr() {
        let listener_sock = UdpSocket::bind("127.0.0.1:0").unwrap();
        let listener_addr = listener_sock.local_addr().unwrap();
        let state = make_state(listener_addr);
        start_listener(listener_sock, state.clone());

        let sender = UdpSocket::bind("127.0.0.1:0").unwrap();
        // node_addr != actual src → should be rejected
        send_announce(
            &sender,
            listener_addr,
            AnnounceMessage {
                node_addr: TaggedSocketAddr::new("127.0.0.1:9999".parse().unwrap()),
                capabilities: vec![],
                recipes: vec![],
                peers: vec![],
                version: v(1),
            },
        );

        std::thread::sleep(Duration::from_millis(100));
        assert_eq!(state.read().unwrap().peers.len(), 0);
    }

    #[test]
    fn listener_discovers_third_party_peers_from_announce() {
        let listener_sock = UdpSocket::bind("127.0.0.1:0").unwrap();
        let listener_addr = listener_sock.local_addr().unwrap();
        let state = make_state(listener_addr);
        start_listener(listener_sock, state.clone());

        let sender = UdpSocket::bind("127.0.0.1:0").unwrap();
        let sender_addr = sender.local_addr().unwrap();
        let third: SocketAddr = "127.0.0.1:9977".parse().unwrap();

        send_announce(
            &sender,
            listener_addr,
            AnnounceMessage {
                node_addr: TaggedSocketAddr::new(sender_addr),
                capabilities: vec![],
                recipes: vec![],
                peers: vec![TaggedSocketAddr::new(third)], // introduces a peer we haven't seen
                version: v(1),
            },
        );

        wait_until(|| state.read().unwrap().get_known_peer(third).is_some());
    }

    #[test]
    fn listener_updates_peer_on_version_change() {
        let listener_sock = UdpSocket::bind("127.0.0.1:0").unwrap();
        let listener_addr = listener_sock.local_addr().unwrap();
        let state = make_state(listener_addr);

        let sender = UdpSocket::bind("127.0.0.1:0").unwrap();
        let sender_addr = sender.local_addr().unwrap();

        // Pre-populate state with an existing peer at version 1.
        state.write().unwrap().peers.push(Peer {
            address: sender_addr,
            capabilities: vec!["old".into()],
            recipes: vec![],
            version: v(1),
            last_seen: 0,
        });
        state.write().unwrap().add_known_peer(sender_addr);

        start_listener(listener_sock, state.clone());

        // Send announce with version 2 and new capabilities.
        send_announce(
            &sender,
            listener_addr,
            AnnounceMessage {
                node_addr: TaggedSocketAddr::new(sender_addr),
                capabilities: vec!["new".into()],
                recipes: vec![],
                peers: vec![],
                version: v(2),
            },
        );

        wait_until(|| {
            state
                .read()
                .unwrap()
                .get_peer(sender_addr)
                .map_or(false, |p| p.version == v(2))
        });
        let s = state.read().unwrap();
        assert_eq!(s.get_peer(sender_addr).unwrap().capabilities, vec!["new"]);
    }
}
