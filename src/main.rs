mod capture;
mod network;

use futures::try_join;
use tokio::runtime;
use tokio::join;
use network::Client;

use std::env;

#[tokio::main]
async fn main() -> () {
  let args: Vec<String> = env::args().collect();
  let mut port: u16 = 4000;
  if args[1] == "-port" {
    port = args[2].parse().unwrap();
  }

  let mut client = Client::new();
  println!("Spawned client {:?} on port {} ", client, port);
  let messager = client.setup_msg(port);
  let capturer = client.capture();

  join!(messager, capturer);
}
