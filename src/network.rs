// The netcode is divided into two parts.
use serde::{Serialize, Deserialize};
use bytes::Bytes;

use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use local_ip_address::local_ip;

use std::sync::{Arc, Mutex};
use std::{thread, time};
use std::collections::VecDeque;
use futures::{SinkExt};

use tokio::net::UdpSocket;
use tokio_util::{udp::UdpFramed, codec::BytesCodec};
use tokio_stream::StreamExt;

use rdev::EventType;
// Replace with enums
// with 2 types: sender, receiver. both holding a udpframed struct

#[derive(Debug, Copy, Clone)]
pub enum State {
    Master,
    Slave
}

// Note!! Peer and state updating could be done through a channel instead
// Client structure, shared amongst thread boundaries of the messaging thread and the capture/network
#[derive(Debug)]
pub struct Client {
    pub state: Arc<Mutex<State>>,
    pub target: Arc<Mutex<usize>>, 
    pub peers: Arc<Mutex<Vec<IpAddr>>>, 
    pub events: VecDeque<EventType>,
}

#[derive(Serialize, Deserialize)]
pub enum Message {
    Discovery,
    DiscoveryResponse,
    TakeControl,
}
// todo: udpframed
// todo: serde
pub static mut EXIT: bool = false;
impl Client {

    // Client Constructor method
    pub fn new() -> Client {
        // All Clients start as master, they downgrade to slave as soon as other peers are found
        Client { state: Arc::new(Mutex::new(State::Master)), target: Arc::new(Mutex::new(0)), peers: Arc::new(Mutex::new(Vec::<IpAddr>::new())), events: VecDeque::new()}
    }
    // Setup the messaging system to find peers
    // and splits it into a different thread
    // TODO: Add port selection argument
    pub async fn setup_msg(&self, capture_port: u16) -> () {

        let msgsocket = UdpSocket::bind("0.0.0.0:4000").await.expect("Cannot bind");
        let localip = local_ip().unwrap();
        println!("Binded on local address {:?} ", localip);
        msgsocket.set_broadcast(true).expect("Unable to set broadcast onser");

        let broadcast_addr : SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(255, 255, 255, 255)), capture_port);
        //msgsocket.send_to(serde_json::to_vec("Hello").unwrap().as_slice(), broadcast_addr).await.unwrap();
        
        let mut msgframe = UdpFramed::new(msgsocket, BytesCodec::new());
        msgframe.send((Bytes::from(serde_json::to_vec(&Message::Discovery).unwrap()), broadcast_addr)).await.unwrap();
        
        let clone_state = self.state.clone();
        let clone_peers = self.peers.clone();
        let clone_tgt = self.target.clone();

        println!("Entering Messaging Loop");
        tokio::spawn(async move 
            {   
                println!("Spawned Messaging Thread");
                loop {
                    println!("Looping");
                    while let Some(Ok((bytes, peer_addr))) = msgframe.next().await {
                        println!("Received: {:?} on {:?} ", bytes, peer_addr);
                        if bytes.len() < 5 || peer_addr.ip() == localip {
                            continue;
                        }
                        let message: Message = serde_json::from_slice::<Message>(&bytes[..]).unwrap();
                        match message {
                            Message::Discovery => {
                                    // Respond to the Discovery with a DiscoveryResponse
                                    msgframe.send((Bytes::from(serde_json::to_vec(&Message::DiscoveryResponse).unwrap()), peer_addr)).await.unwrap();
                                    thread::sleep(time::Duration::from_millis(50));
                                    // Send peer_addr to vectors of peers
                                    println!("Added peer {} to the list, from Discovery ping", peer_addr);
                                    clone_peers.lock().unwrap().push(peer_addr.ip());
                                    // Technically unsafe :D
                                    let mut cstate = State::Slave; 
                                    {
                                        let state = clone_state.lock().unwrap();
                                        cstate = *state;
                                    }
                                    /*
                                    match cstate {
                                        State::Slave => { ;/* Do nothing */},
                                        State::Master => { msgframe.send((Bytes::from(serde_json::to_vec(&Message::TakeControl).unwrap()), peer_addr)).await.unwrap(); },
                                    }*/
                                },
                            Message::DiscoveryResponse => {
                                    // Send peer_addr to vectors of peers
                                    clone_peers.lock().unwrap().push(peer_addr.ip());
                                    
                                    // Change state to slave if other peers are on the network (only possible if DiscResponse is received)
                                    let mut state = clone_state.lock().unwrap();
                                    *state = State::Slave;
                                },
                            Message::TakeControl => {
                                    // Transmute the client and 
                                    // If a slave receives a takecontrol command, transfer ownership of that slave to the new addr
                                    // to do so, change target value to match the index of the Message::TakeControl sender
                                    let mut state = clone_state.lock().unwrap();
                                    *state = State::Slave;
                                    
                                    if let Some(index) = clone_peers.lock().unwrap().iter().position(|&addr| addr == peer_addr.ip()) {
                                        *(clone_tgt.lock().unwrap()) = index;
                                    } else {
                                        clone_peers.lock().unwrap().push(peer_addr.ip());
                                    }
                                    println!("Changed control");
                                }
                        }
                        unsafe {
                            if EXIT == true {
                                println!("Cancelling task");
                                return;
                            }
                        }  
                    }  
                }
            }).await.unwrap();
    }
}