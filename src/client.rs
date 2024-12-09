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

use std::sync::{Arc, Mutex};

#[derive(Debug, Default, Clone)]
struct ClientState {
    num_write_shards: usize,
    write_shard_info: Vec<(String, u16)>,
    read_shard_info: Vec<(String, u16)>,
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
    fn handle_write_request(&self, req: &WriteRequest) -> WriteResponse {
        println!(
            "Sending write request to server: key = {}, value = {}",
            String::from_utf8_lossy(&req.key),
            String::from_utf8_lossy(&req.value)
        );
        WriteResponse { error: 0 } // Simulated response
    }

    fn handle_read_request(&self, req: &ReadRequest) -> ReadResponse {
        println!(
            "Sending read request to server: key = {}",
            String::from_utf8_lossy(&req.key)
        );
        ReadResponse {
            value: b"Simulated value".to_vec(),
        } // Simulated response
    }

    fn handle_query_version_request(&self, _req: &QueryVersionRequest) -> QueryVersionResponse {
        unimplemented!()
    }

    fn handle_get_version_request(&self, _req: &GetVersionRequest) -> GetVersionResponse {
        unimplemented!()
    }

    // Unimplemented methods
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
    fn handle_get_client_shard_info_response(&self, _res: &GetClientShardInfoResponse) {
        unimplemented!()
    }
    fn handle_query_version_response(&self, _res: &QueryVersionResponse) {
        unimplemented!()
    }
    fn handle_read_response(&self, _res: &ReadResponse) {
        unimplemented!()
    }
    fn handle_write_response(&self, _res: &WriteResponse) {
        unimplemented!()
    }
    fn handle_get_shared_peers_response(&self, _res: &GetSharedPeersResponse) {
        unimplemented!()
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
    let client_router = RouterBuilder::new(
        Client::new(main_info_server.to_string(), Arc::clone(&shard_state)),
        None,
    );

    // Periodically fetch shard info
    tokio::spawn({
        async move {
            loop {
                let request = GetClientShardInfoRequest {}; // Assuming this struct exists
                if let Err(err) = client_router
                    .queue_request(request, main_info_server.to_string())
                    .await
                {
                    eprintln!("Failed to fetch shard info: {}", err);
                }
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            }
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
                        let (ip, port) = shard_state_lock.write_shard_info[shard_index].clone();
                        target = format!("{}:{}", ip, port);
                    } // Lock is released here when `shard_state_lock` goes out of scope

                    let request = WriteRequest {
                        key: key.as_bytes().to_vec(),
                        value: value.as_bytes().to_vec(),
                    };

                    // Use `Arc::clone(&shard_state)` directly
                    let client_router = RouterBuilder::new(
                        Client::new(target.clone(), Arc::clone(&shard_state)),
                        None,
                    );

                    if let Err(err) = client_router.queue_request(request, target).await {
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
                    // Lock the shard state to read configuration
                    let shard_index;
                    let target;
                    {
                        let shard_state_lock = shard_state.lock().unwrap();
                        shard_index = hash_key_to_shard(key, shard_state_lock.num_write_shards);
                        let (ip, port) = shard_state_lock.read_shard_info[shard_index].clone();
                        target = format!("{}:{}", ip, port);
                    } // Lock is released here when `shard_state_lock` goes out of scope

                    let request = ReadRequest {
                        key: key.as_bytes().to_vec(),
                    };

                    // Use `Arc::clone(&shard_state)` directly
                    let client_router = RouterBuilder::new(
                        Client::new(target.clone(), Arc::clone(&shard_state)),
                        None,
                    );

                    if let Err(err) = client_router.queue_request(request, target).await {
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
