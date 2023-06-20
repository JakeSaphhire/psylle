use futures::{SinkExt, TryFutureExt};
use rdev::{grab, simulate, Event, EventType, Key, SimulateError};

use serde::{Serialize, Deserialize};
use postcard::{from_bytes, to_vec, to_allocvec};
use bytes::Bytes;

use tokio::task::JoinHandle;
use tokio::net::UdpSocket;
use tokio_util::{udp::UdpFramed, codec::BytesCodec};
use tokio_stream::StreamExt;

use std::sync::mpsc;
use std::cell::RefCell;
use std::{thread, time};
use crate::network::{Client, Message, State};


impl Client {
    // must be called after messaging is setup
    pub async fn capture(&mut self) -> () {
        // See https://github.com/Narsil/rdev/issues/101#issuecomment-1500698317 and 
        // and https://github.com/jersou/mouse-actions/blob/7bd717d32408d1b836e031531f1d051b51957e04/src/main.rs#L33
        thread::sleep(time::Duration::from_millis(300));
        static mut EXIT: bool = false;

        // Seup main socket
        let mainsocket = UdpSocket::bind("127.0.0.1:4001").await.expect("Cannot bind main socket");
        let mut mainframe = UdpFramed::new(mainsocket, BytesCodec::new());

        let (tx, rx) = mpsc::channel();
        
        loop {
            // Alternative! Use listen instead of grab...
            let callback_tx = tx.clone();
            let callback = move | event: Event| -> Option<Event> {
                match event.event_type {
                    EventType::KeyPress(Key::Escape) => {unsafe{EXIT = true}; Some(event)},
                    _ => {
                        callback_tx.send(event.event_type).unwrap();             
                        None
                    },
                }
            }; 
            match *self.state.lock().unwrap() {
                State::Master => {
                    if let Err(e) = grab(callback) {
                        println!("Grabbing error: {:?}", e);
                    }
                    unsafe{
                        if EXIT == true {()}
                    }
                    while let Ok(event) = rx.recv() {
                        // Send event on the network immediately
                        let index = *self.target.lock().unwrap();
                        let peers = self.peers.lock().unwrap();
                        mainframe.send((Bytes::from(to_allocvec(&event).unwrap()), peers[index])).await.unwrap();
                    }
                },
                State::Slave => {
                    if let Some(Ok((bytes, _peer_addr))) = mainframe.next().await {
                        //  Deserialize and Simulate the event
                        // send(&event) etc...
                        let del = time::Duration::from_millis(20);
                        match simulate(&from_bytes::<Event>(&bytes[..]).unwrap().event_type) {
                            Ok(()) => (), 
                            Err(SimulateError) => {
                                println!("Could not send event");
                            }
                        }
                        thread::sleep(del);
                    }
                }, 
            }   
        }
    }
}