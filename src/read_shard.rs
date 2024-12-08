use anyhow::{Ok, Result};
use rand::Rng;
use std::collections::HashMap;
use std::net::SocketAddr;
use tokio::{
    net::{TcpListener, TcpStream},
    time::{sleep, Duration},
};
mod io;
use io::read_write::{read_message, write_message};

mod messages;

static MAIN_INSTANCE_IP_PORT: ([u8; 16], u16) =
    ([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], 8080);

pub struct ReadShard {
    reader_ip_port: SocketAddr,
    writer_id: u16,
    writer_ip_port: ([u8; 16], u16),
    peers: Vec<([u8; 16], u16)>,
    current_version: u64,
    history: Vec<(String, String)>,
    data: HashMap<String, String>,
}

impl ReadShard {
    pub fn new(
        reader_ip_port: SocketAddr,
        writer_ip_port: ([u8; 16], u16),
        writer_id: u16,
    ) -> ReadShard {
        ReadShard {
            reader_ip_port,
            writer_id,
            writer_ip_port,
            peers: Vec::new(),
            current_version: 0,
            history: Vec::new(),
            data: HashMap::new(),
        }
    }

    pub async fn announce_shard(&mut self) -> Result<()> {
        // Sends its ip port

        // Gets a response
        let response: u16 = 1;

        self.writer_id = response;

        Ok(())
    }

    pub async fn update_shard_from_peers(&mut self) -> Result<()> {
        loop {
            // TODO: get list of ports and update the peers list
            let mut rnd = rand::thread_rng();
            let update_shard_addr = self.peers[rnd.gen_range(0..self.peers.len())];

            // TODO: request for version
            let response_version = 1;
            if response_version > self.current_version {
                // TODO: request for data of the next version from the shard
                let response = ("key".to_string(), "value".to_string());
                let response_clone = response.clone();

                self.data.insert(response.0, response.1);
                self.history.push(response_clone);
                self.current_version += 1
            }
            sleep(Duration::from_secs(2)).await;
        }
    }

    pub async fn get_value(&self, key: &str) -> Option<&String> {
        self.data.get(&key.to_string())
    }
}

pub fn socket_addr_to_string(addr: ([u8; 16], u16)) -> String {
    format!(
        "{}:{}",
        addr.0
            .iter()
            .map(|x| x.to_string())
            .collect::<Vec<String>>()
            .join("."),
        addr.1
    )
}

#[tokio::main]
async fn main() -> Result<()> {
    let listener = TcpListener::bind("127.0.0.1:0").await?;

    let reader_addr = listener.local_addr().unwrap();
    // Request for peers addr

    let writer_addr = ([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], 8080);
    let writer_num = 1;

    let mut reader_shard = ReadShard::new(reader_addr, writer_addr, writer_num);

    tokio::spawn(async move {
        reader_shard.announce_shard().await.unwrap();
    });

    tokio::spawn(async move {
        reader_shard.update_shard_from_peers().await.unwrap();
    });

    // Check if request asks to read and then send value to client
    tokio::spawn(async move {
        reader_shard.get_value("key");
    });

    Ok(())
}
