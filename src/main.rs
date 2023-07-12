mod capture;
mod network;

use tokio::join;
use network::Client;

use std::env;

/*
  TODO!! 
  0. Switch .send() to udpsockets
  1. Toggling remote control
  2. Target selection
  3. Optimisation
   
 */

const SSO: usize = 1920;

#[tokio::main]
async fn main() -> () {
  let args: Vec<String> = env::args().collect();
  let mut port: u16 = 4000;
  if args[1] == "-port" {
    port = args[2].parse().unwrap();
  }

  let client = Client::new();
  println!("Spawned client {:?} on port {} ", client, port);
  let messager = client.setup_msg(port);
  let capturer = client.capture();

  join!(messager, capturer);
}
