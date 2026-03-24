use std::net::{SocketAddr, UdpSocket};
use crate::gossip::receiver::add_known_peer;
use crate::gossip::state::SharedGossipState;

pub enum Command {
    AddPeer{socket_addr: SocketAddr}    ,
}
pub(super) fn start_command_listener(
    shared_gossip_state: SharedGossipState,
    edit_receiver: std::sync::mpsc::Receiver<Command>,
    udp_socket: &UdpSocket
) {
    let udp_socket = udp_socket.try_clone().expect("Failed to clone UDP socket");
    std::thread::spawn(move || {
        while let Ok(edit) = edit_receiver.recv() {
            match edit {
                Command::AddPeer { socket_addr } => {
                    add_known_peer( socket_addr, &shared_gossip_state, &udp_socket);
                }
            }
        }
    });
}
