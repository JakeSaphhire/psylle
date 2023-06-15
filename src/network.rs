// The netcode is divided into two parts.
use serde::{Serialize, Deserialize};
use postcard::{from_bytes, to_vec, to_allocvec};
use bytes::Bytes;

use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::io;
use std::ops::Deref;
use std::sync::{Arc, mpsc, Mutex};
use std::collections::VecDeque;
use futures::{FutureExt, SinkExt};

use tokio::task::JoinHandle;
use tokio::net::UdpSocket;
use tokio_util::{udp::UdpFramed, codec::BytesCodec};
use tokio_stream::StreamExt;

use rdev::EventType;
// Replace with enums
// with 2 types: sender, receiver. both holding a udpframed struct

#[derive(Debug, Copy, Clone)]
enum State {
    Master,
    Slave
}

// Note!! Peer and state updating could be done through a channel instead
// Client structure, shared amongst thread boundaries of the messaging thread and the capture/network
pub struct Client {
    state: Arc<Mutex<State>>,
    target: Arc<Mutex<usize>>, 
    peers: Arc<Mutex<Vec<SocketAddr>>>, 
    events: VecDeque<EventType>,
}

#[derive(Serialize, Deserialize)]
pub enum Message {
    Discovery,
    DiscoveryResponse,
    TakeControl,
}
// todo: udpframed
// todo: serde

impl Client {

    // Client Constructor method
    async fn new() -> Client {
        // All Clients start as master, they downgrade to slave as soon as other peers are found
        Client { state: Arc::new(Mutex::new(State::Master)), target: Arc::new(Mutex::new(0)), peers: Arc::new(Mutex::new(Vec::<SocketAddr>::new())), events: VecDeque::new()}
    }
    // Setup the messaging system to find peers
    // and splits it into a different thread
    async fn setup_msg(&self) -> JoinHandle<()> {

        let msgsocket = UdpSocket::bind("127.0.0.1:4000").await.expect("cannot bind free");
        let broadcast_addr : SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(255, 255, 255, 255)), 4000);
        msgsocket.set_broadcast(true).expect("Unable to set broadcast onser");
        
        let mut msgframe = UdpFramed::new(msgsocket, BytesCodec::new());
        msgframe.send((Bytes::from(to_allocvec(&Message::Discovery).unwrap()), broadcast_addr)).await.unwrap();
        
        let clone_state = self.state.clone();
        let clone_peers = self.peers.clone();
        let clone_tgt = self.target.clone();
        tokio::spawn(async move 
            {
                while let Some(Ok((bytes, peer_addr))) = msgframe.next().await {
                    let message: Message = from_bytes::<Message>(&bytes[..]).expect("Failed to deserialize");
                    match message {
                        Message::Discovery => {
                                // Respond to the Discovery with a DiscoveryResponse
                                msgframe.send((Bytes::from(to_allocvec(&Message::DiscoveryResponse).unwrap()), peer_addr)).await.unwrap();
                                 
                                // Technically unsafe :D
                                let mut cstate = State::Slave; 
                                {
                                    let state = clone_state.lock().unwrap();
                                    cstate = *state;
                                }

                                match cstate {
                                    State::Slave => { /* Do nothing */},
                                    State::Master => { msgframe.send((Bytes::from(to_allocvec(&Message::DiscoveryResponse).unwrap()), peer_addr)).await.unwrap(); },
                                }
                            },
                        Message::DiscoveryResponse => {
                                // Send peer_addr to vectors of peers
                                clone_peers.lock().unwrap().push(peer_addr);
                                
                                // Change state to slave if other peers are on the network (only possible if DiscResponse is received)
                                let mut state = clone_state.lock().unwrap();
                                *state = State::Slave;
                            },
                        Message::TakeControl => {
                                // Transmute the client and 
                                // If a slave receives a takecontrol command, transfer ownership of that slave to the new addr
                                // to do so, change target value to match the index of the Message::TakeControl sender
                                let mut state = clone_state.lock().unwrap();
                                match *state {
                                    State::Slave => { /* Do Nothing */},
                                    State::Master => {*state = State::Slave},
                                }
                                drop(state);
                                
                                if let Some(index) = clone_peers.lock().unwrap().iter().position(|&addr| addr == peer_addr) {
                                    *(clone_tgt.lock().unwrap()) = index;
                                } else {
                                    clone_peers.lock().unwrap().push(peer_addr);
                                }
                                
                            }
                    }
                }
            })
    }

    pub fn process()  -> () {

    }
}