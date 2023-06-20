mod capture;
mod network;

use tokio::runtime;
use network::Client;

use std::env;

#[tokio::main]
async fn main() -> () {
  let args: Vec<String> = env::args().collect();
  let mut port: u16 = 4000;
  if args[1] == "--p" {
    port = args[2].parse().unwrap();
  }

  let mut client = Client::new();
  client.setup_msg(port).await;
  client.capture().await;
}
