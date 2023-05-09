// The netcode is divided into two parts.
use std::collections::VecDeque;
use rdev::EventType;
pub struct client {
    master: bool, // Transitting or receiving
    friends: Vec<SocketAddr>,
    target: SocketAddr,
    recv_queue: VecDeque<EventType>,
    send_queue: VecDeque<EventType>
}

use std::{io, net::SocketAddr};
pub fn out_client() -> io::Result<()> {

    Ok(())
}