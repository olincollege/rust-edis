pub mod io;
pub mod messages;

use crate::io::router::{RouterBuilder, RouterHandler};
use anyhow::Result;
use messages::{
    requests::{
        announce_shard_request::AnnounceShardRequest,
        get_client_shard_info_request::GetClientShardInfoRequest,
        get_shared_peers_request::GetSharedPeersRequest, get_version_request::GetVersionRequest,
        query_version_request::QueryVersionRequest, read_request::ReadRequest,
        write_request::WriteRequest,
    },
    responses::{
        announce_shard_response::AnnounceShardResponse,
        get_client_shard_info_response::GetClientShardInfoResponse,
        get_shared_peers_response::GetSharedPeersResponse,
        get_version_response::GetVersionResponse, query_version_response::QueryVersionResponse,
        read_response::ReadResponse, write_response::WriteResponse,
    },
};
use std::io::Write;

use std::net::{Ipv6Addr, SocketAddrV6};
use std::sync::{Arc, Mutex};

#[derive(Debug, Default, Clone)]
struct ClientState {
    num_write_shards: usize,
    write_shard_info: Vec<SocketAddrV6>,
    read_shard_info: Vec<SocketAddrV6>,
}

#[derive(Debug, Clone)]
struct Client {
    server_address: String,
    shard_state: Arc<Mutex<ClientState>>,
}

impl Client {
    fn new(server_address: String, shard_state: Arc<Mutex<ClientState>>) -> Self {
        Client {
            server_address,
            shard_state,
        }
    }
}

impl RouterHandler for Client {
    fn handle_write_request(&self, _req: &WriteRequest) -> WriteResponse {
        unimplemented!()
    }

    fn handle_read_request(&self, _req: &ReadRequest) -> ReadResponse {
        unimplemented!()
    }

    fn handle_get_client_shard_info_response(&self, res: &GetClientShardInfoResponse) {
        println!("Received shard information from main info server:");

        let num_write_shards = res.num_write_shards as usize;
        let write_shard_info: Vec<SocketAddrV6> = res
            .write_shard_info
            .iter()
            .map(|(ip, port)| SocketAddrV6::new(Ipv6Addr::from(*ip), *port, 0, 0))
            .collect();

        let read_shard_info: Vec<SocketAddrV6> = res
            .read_shard_info
            .iter()
            .map(|(ip, port)| SocketAddrV6::new(Ipv6Addr::from(*ip), *port, 0, 0))
            .collect();

        println!("Number of write shards: {}", num_write_shards);
        println!("Write Shards: {:?}", write_shard_info);
        println!("Read Shards: {:?}", read_shard_info);

        // Update the client state
        let mut shard_state = self.shard_state.lock().unwrap();
        shard_state.num_write_shards = num_write_shards;
        shard_state.write_shard_info = write_shard_info;
        shard_state.read_shard_info = read_shard_info;
    }

    fn handle_write_response(&self, res: &WriteResponse) {
        match res.error {
            0 => println!("Write operation successful."),
            _ => eprintln!("Write operation failed with error code: {}", res.error),
        }
    }

    fn handle_read_response(&self, res: &ReadResponse) {
        if res.value.is_empty() {
            println!("Key not found or value is empty.");
        } else {
            println!(
                "Read operation successful. Key: {}, Value: {}",
                String::from_utf8_lossy(&res.key),
                String::from_utf8_lossy(&res.value)
            );
        }
    }

    fn handle_query_version_request(&self, _req: &QueryVersionRequest) -> QueryVersionResponse {
        unimplemented!()
    }

    fn handle_get_version_request(&self, _req: &GetVersionRequest) -> GetVersionResponse {
        unimplemented!()
    }

    fn handle_announce_shard_request(&self, _req: &AnnounceShardRequest) -> AnnounceShardResponse {
        unimplemented!()
    }
    fn handle_get_client_shard_info_request(
        &self,
        _req: &GetClientShardInfoRequest,
    ) -> GetClientShardInfoResponse {
        unimplemented!()
    }
    fn handle_get_shared_peers_request(
        &self,
        _req: &GetSharedPeersRequest,
    ) -> GetSharedPeersResponse {
        unimplemented!()
    }
    fn handle_announce_shard_response(&self, _res: &AnnounceShardResponse) {
        unimplemented!()
    }

    fn handle_query_version_response(&self, _res: &QueryVersionResponse) {
        unimplemented!()
    }

    fn handle_get_shared_peers_response(&self, res: &GetSharedPeersResponse) {
        let mut peers = self.shard_state.lock().unwrap();
        peers.write_shard_info = res
            .peer_ips
            .iter()
            .map(|(ip, port)| SocketAddrV6::new(Ipv6Addr::from(*ip), *port, 0, 0))
            .collect();
    }

    fn handle_get_version_response(&self, _res: &GetVersionResponse) {
        unimplemented!()
    }
}

fn hash_key_to_shard(key: &str, num_shards: usize) -> usize {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    key.hash(&mut hasher);
    (hasher.finish() as usize) % num_shards
}

#[tokio::main]
async fn main() -> Result<()> {
    let main_info_server = "127.0.0.1:8080";

    // Create shared state
    let shard_state = Arc::new(Mutex::new(ClientState::default()));
    let client_router = Arc::new(RouterBuilder::new(
        Client::new(main_info_server.to_string(), Arc::clone(&shard_state)),
        None,
    ));

    let main_info_server = SocketAddrV6::new(Ipv6Addr::LOCALHOST, 8080, 0, 0);

    let client_router_clone = Arc::clone(&client_router);
    tokio::spawn(async move {
        loop {
            let request = GetClientShardInfoRequest {};
            if let Err(err) = client_router_clone
                .get_router_client()
                .queue_request::<GetClientShardInfoRequest>(request, main_info_server)
                .await
            {
                eprintln!("Failed to fetch shard info: {}", err);
            }
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        }
    });

    println!(
        "Connected to WriteShard database through Main Info Server at {}",
        main_info_server
    );
    println!("Available commands: set <key> <value>, get <key>, exit");

    loop {
        print!("> ");
        std::io::stdout().flush().unwrap();

        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();
        let input = input.trim();

        let mut parts = input.splitn(3, ' ');
        let command = parts.next().unwrap_or("").to_lowercase();
        let key = parts.next();
        let value = parts.next();

        match command.as_str() {
            "set" => {
                if let (Some(key), Some(value)) = (key, value) {
                    // Lock the shard state to read configuration
                    let shard_index;
                    let target;
                    {
                        let shard_state_lock = shard_state.lock().unwrap();
                        shard_index = hash_key_to_shard(key, shard_state_lock.num_write_shards);
                        target = shard_state_lock.write_shard_info[shard_index];
                    } // Lock is released here when `shard_state_lock` goes out of scope

                    let request = WriteRequest {
                        key: key.as_bytes().to_vec(),
                        value: value.as_bytes().to_vec(),
                    };

                    let router_client = client_router.get_router_client();
                    if let Err(err) = router_client
                        .queue_request::<WriteRequest>(request, target)
                        .await
                    {
                        eprintln!("Failed to queue write request: {}", err);
                    } else {
                        println!("OK");
                    }
                } else {
                    println!("Usage: set <key> <value>");
                }
            }
            "get" => {
                if let Some(key) = key {
                    let shard_index;
                    let target;
                    {
                        let shard_state_lock = shard_state.lock().unwrap();
                        shard_index = hash_key_to_shard(key, shard_state_lock.num_write_shards);
                        target = shard_state_lock.read_shard_info[shard_index];
                    }

                    let request = ReadRequest {
                        key: key.as_bytes().to_vec(),
                    };

                    let router_client = client_router.get_router_client();
                    if let Err(err) = router_client
                        .queue_request::<ReadRequest>(request, target)
                        .await
                    {
                        eprintln!("Failed to queue read request: {}", err);
                    } else {
                        println!(
                            "Request queued successfully. Check the server logs for the response."
                        );
                    }
                } else {
                    println!("Usage: get <key>");
                }
            }
            "exit" => {
                println!("Goodbye!");
                break;
            }
            _ => {
                println!("Unknown command. Available commands: set, get, exit");
            }
        }
    }

    Ok(())
}
