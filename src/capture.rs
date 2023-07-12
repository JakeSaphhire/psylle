
use rdev::{simulate, Event, EventType, Key, SimulateError};

use std::net::SocketAddr;
use tokio::net::UdpSocket;

use tokio::sync::mpsc;
use std::{thread, time, process};
use std::time::{Duration, SystemTime};
use crate::network::{Client, State, EXIT};
use crate::SSO;      


impl Client {
    // must be called after messaging is setup
    pub async fn capture(&self) -> () {
        // See https://github.com/Narsil/rdev/issues/101#issuecomment-1500698317 and 
        // and https://github.com/jersou/mouse-actions/blob/7bd717d32408d1b836e031531f1d051b51957e04/src/main.rs#L33
        thread::sleep(time::Duration::from_millis(300));

        // Seup main socket
        let mainsocket = UdpSocket::bind("0.0.0.0:4001").await.expect("Cannot bind main socket");

        let (tx, mut rx) = mpsc::unbounded_channel();

        println!("Spawning Capture Thread");
        // Alternative! Use listen instead of grab...
        let callback_tx = tx.clone();
        static mut PAUSE: bool = false;
        thread::spawn(move || {
            rdev::listen(move | mut event: Event| -> () {
                unsafe {
                    match (event.event_type, EXIT, PAUSE) {
                        (EventType::KeyRelease(Key::ScrollLock), false, _) => {PAUSE = !PAUSE;},
                        (EventType::KeyRelease(Key::Escape), false, _) => {EXIT = true;},
                        (EventType::MouseMove { x, y }, false, false) => {
                            event.event_type  = rdev::EventType::MouseMove {x: x, y: y - 2.0*SSO as f64};
                            callback_tx.send(event.clone()).unwrap();
                        },
                            
                        (_, false, false) => {
                            // println!("Grabbed {:?} ", event);
                            callback_tx.send(event.clone()).unwrap();
                        },
                        (_, true, _) => {process::exit(0); }
                        (_, _, true) => {}
                    }
                }
            }).unwrap();
        });
        
        let mut last_time =  SystemTime::now();
        loop {
            println!("Looping");
            match *self.state.lock().unwrap() {
                State::Master => {
                    if let Some(event) = rx.recv().await {
                        println!(" {:?} on the loop thread", event);
                        // Send event on the network immediately
                        // Skip sending if it's from less than 1ms ago
                        if last_time.elapsed().unwrap() <= Duration::from_millis(2)
                        {
                            continue;
                        }
                        let index = *self.target.lock().unwrap();
                        let peers = self.peers.lock().unwrap();
                        println!("Sending {:?} to {} ", serde_json::to_string(&event).unwrap(), peers[index]);
                        let _ = mainsocket.try_send_to(&serde_json::to_vec(&event).unwrap()[..], SocketAddr::new(peers[index], 4001));
                        last_time = event.time;
                        
                    }
                },
                State::Slave => {
                    let mut receive_buffer = vec![0u8; 1024];
                    if let Ok((_size, _peer_addr)) = mainsocket.try_recv_from(&mut receive_buffer) {
                        // Deserialize and Simulate the event
                        // send(&event) etc...
                        match simulate(&serde_json::from_slice::<Event>(&receive_buffer[..]).unwrap().event_type) {
                            Ok(()) => (), 
                            Err(SimulateError) => {
                                println!("Could not send event");
                            }
                        }
                    }
                }, 
            }
            unsafe {
                if EXIT == true {
                    return;
                }
            }
        }
        
    }
}