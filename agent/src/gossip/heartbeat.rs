use crate::gossip::message::{GossipError, Message, PingMessage, PongMessage, send_message};
use crate::gossip::state::{Peer, SharedGossipState};
use std::net::{SocketAddr, UdpSocket};
use std::thread;
use std::time::{Duration, SystemTime};

const REFRESH_TIMEOUT: Duration = Duration::from_secs(10);
const REFRESH_DELAY: Duration = Duration::from_secs(1);

#[derive(Clone, Copy)]
pub(super) struct PingLoopConfig {
    refresh_timeout: Duration,
    refresh_delay: Duration,
}

impl Default for PingLoopConfig {
    fn default() -> Self {
        Self {
            refresh_timeout: REFRESH_TIMEOUT,
            refresh_delay: REFRESH_DELAY,
        }
    }
}

pub(super) fn ping_loop(
    address: SocketAddr,
    shared_gossip_state: SharedGossipState,
    udp_socket: &UdpSocket,
    config: PingLoopConfig,
) -> Result<(), GossipError> {
    loop {
        let (last_seen, version) = {
            let state = shared_gossip_state.read().expect("poisoned lock");
            match state.get_peer(address) {
                Some(p) => (p.last_seen, p.version),
                // Peer was removed (e.g. by another ping_loop instance or remove_peer); exit cleanly.
                None => return Ok(()),
            }
        };

        let last_seen_time = SystemTime::UNIX_EPOCH + Duration::from_millis(last_seen as u64);
        if last_seen_time.elapsed().unwrap_or(Duration::ZERO) > config.refresh_timeout {
            shared_gossip_state
                .write()
                .expect("poisoned lock")
                .remove_peer(address);
            return Ok(());
        }

        send_message(
            udp_socket,
            address,
            &Message::Ping(PingMessage { last_seen, version }),
        )?;

        thread::sleep(config.refresh_delay);
    }
}

pub(super) fn send_pong(
    peer: &Peer,
    shared_gossip_state: &SharedGossipState,
    udp_socket: &UdpSocket,
) -> Result<(), GossipError> {
    let version = shared_gossip_state.read().expect("poisoned lock").version;
    let pong_msg = Message::Pong(PongMessage {
        last_seen: peer.last_seen,
        version,
    });
    send_message(udp_socket, peer.address, &pong_msg)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gossip::state::{GossipState, SharedGossipState};
    use crate::gossip::version::Version;
    use std::net::UdpSocket;
    use std::sync::{Arc, RwLock};
    use std::time::{SystemTime, UNIX_EPOCH};

    fn addr(port: u16) -> SocketAddr {
        format!("127.0.0.1:{port}").parse().unwrap()
    }

    fn v1() -> Version {
        Version {
            counter: 1,
            generation: 0,
        }
    }

    fn now_ms() -> u128 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis()
    }

    fn make_state(port: u16) -> SharedGossipState {
        Arc::new(RwLock::new(GossipState::new(addr(port))))
    }

    fn push_peer(state: &SharedGossipState, peer_addr: SocketAddr, last_seen: u128) {
        state.write().unwrap().peers.push(Peer {
            address: peer_addr,
            capabilities: vec![],
            recipes: vec![],
            version: v1(),
            last_seen,
        });
    }

    #[test]
    fn ping_loop_exits_when_peer_absent() {
        let state = make_state(0);
        let socket = UdpSocket::bind("127.0.0.1:0").unwrap();
        // Peer is not in state.peers → loop must exit immediately without sending.
        ping_loop(addr(19001), state, &socket, Default::default()).unwrap();
    }

    #[test]
    fn ping_loop_removes_stale_peer_and_exits() {
        let state = make_state(0);
        let peer_socket = UdpSocket::bind("127.0.0.1:0").unwrap();
        let peer_addr = peer_socket.local_addr().unwrap();

        // last_seen = 0 -> not seen since UNIX_EPOCH
        push_peer(&state, peer_addr, 0);
        assert!(state.read().unwrap().get_peer(peer_addr).is_some());

        let socket = UdpSocket::bind("127.0.0.1:0").unwrap();
        ping_loop(peer_addr, state.clone(), &socket, Default::default()).unwrap();
        assert!(state.read().unwrap().get_peer(peer_addr).is_none());
    }

    #[test]
    fn ping_loop_sends_ping_with_correct_timeout() {
        let state = make_state(0);
        let peer_socket = UdpSocket::bind("127.0.0.1:0").unwrap();
        let peer_addr = peer_socket.local_addr().unwrap();

        push_peer(&state, peer_addr, now_ms());
        assert!(state.read().unwrap().get_peer(peer_addr).is_some());

        let socket = UdpSocket::bind("127.0.0.1:0").unwrap();
        let config = PingLoopConfig {
            refresh_timeout: Duration::from_secs(1),
            refresh_delay: Duration::from_millis(200),
        };
        let start = SystemTime::now();
        ping_loop(peer_addr, state.clone(), &socket, config).unwrap();
        let elapsed = start.elapsed().unwrap();
        assert!(elapsed >= config.refresh_timeout);
        assert!(elapsed < config.refresh_timeout + Duration::from_millis(500)); // Allow some leeway for timing inaccuracy
    }

    #[test]
    fn ping_loop_sends_ping_on_interval() {
        let state = make_state(0);
        let peer_socket = UdpSocket::bind("127.0.0.1:0").unwrap();
        let peer_addr = peer_socket.local_addr().unwrap();
        let config = PingLoopConfig {
            refresh_timeout: Duration::from_secs(5),
            refresh_delay: Duration::from_millis(200),
        };

        push_peer(&state, peer_addr, now_ms());
        assert!(state.read().unwrap().get_peer(peer_addr).is_some());

        let start = SystemTime::now();

        let handle = thread::spawn(move || {
            let mut ping_count = 0;
            let timeout = Duration::from_millis(700);
            peer_socket.set_read_timeout(Some(timeout)).unwrap();
            let mut buf = [0u8; 1024];
            loop {
                match peer_socket.recv_from(&mut buf) {
                    Ok((len, _)) => {
                        let msg: Message = ciborium::de::from_reader(&buf[..len]).unwrap();
                        if let Message::Ping(p) = msg {
                            assert_eq!(p.version, v1());
                            if start.elapsed().unwrap() < timeout {
                                ping_count += 1;
                            } else {
                                break;
                            }
                        }
                    }
                    Err(e) => panic!("Unexpected socket error: {e}"),
                }
            }
            assert_eq!(ping_count, 4); // Expect ~4 pings in 700ms with 200ms interval
        });
        thread::spawn({
            let state = state.clone();
            let socket = UdpSocket::bind("127.0.0.1:0").unwrap();
            move || ping_loop(peer_addr, state, &socket, config).unwrap()
        });

        handle.join().expect("ping loop thread panicked");
    }

    #[test]
    fn send_pong_delivers_pong_message() {
        let state = make_state(0);
        let sender = UdpSocket::bind("127.0.0.1:0").unwrap();
        let receiver = UdpSocket::bind("127.0.0.1:0").unwrap();
        receiver
            .set_read_timeout(Some(Duration::from_secs(1)))
            .unwrap();

        let peer = Peer {
            address: receiver.local_addr().unwrap(),
            capabilities: vec![],
            recipes: vec![],
            version: v1(),
            last_seen: now_ms(),
        };

        send_pong(&peer, &state, &sender).unwrap();

        let mut buf = [0u8; 1024];
        let (len, _) = receiver.recv_from(&mut buf).unwrap();
        let msg: Message = ciborium::de::from_reader(&buf[..len]).unwrap();
        assert!(matches!(msg, Message::Pong(_)));
    }
}
