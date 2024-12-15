use anyhow::Result;
use io::write;
use messages::requests::announce_shard_request::ShardType;
use rand::Rng;
use std::collections::HashMap;
use std::net::{Ipv6Addr, SocketAddrV6};
use std::sync::{Arc, Mutex};
use tokio::time;
mod integration;
mod messages;
mod utils;
use crate::messages::{
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
        get_version_response::{GetVersionResponse, GetVersionResponseError},
        query_version_response::QueryVersionResponse,
        read_response::ReadResponse,
        write_response::WriteResponse,
    },
};
mod io;
use io::router::{RouterBuilder, RouterHandler};

static MAIN_INSTANCE_IP_PORT: SocketAddrV6 = SocketAddrV6::new(Ipv6Addr::LOCALHOST, 8080, 0, 0);

#[derive(Debug)]
struct WriteShard {
    data: Arc<Mutex<HashMap<String, String>>>,
    version_history: Arc<Mutex<Vec<(String, String)>>>,
    current_version: Arc<Mutex<u64>>,
}

impl WriteShard {
    fn new() -> Self {
        WriteShard {
            data: Arc::new(Mutex::new(HashMap::new())),
            version_history: Arc::new(Mutex::new(Vec::new())),
            current_version: Arc::new(Mutex::new(0)),
        }
    }
}

impl RouterHandler for WriteShard {
    fn handle_write_request(&self, req: &WriteRequest) -> WriteResponse {
        // Extract key and value from the request
        let key = String::from_utf8(req.key.clone()).unwrap();
        let value = String::from_utf8(req.value.clone()).unwrap();

        // Lock and increment the current version
        let mut current_version = self.current_version.lock().unwrap();
        *current_version += 1;

        // Lock and update the data
        let mut data = self.data.lock().unwrap();
        data.insert(key.clone(), value.clone());

        // Lock and update the version history
        let mut version_history = self.version_history.lock().unwrap();
        version_history.push((key.clone(), value.clone()));
        println!(
            "wrote key: {}, value: {}, version: {}",
            key, value, *current_version
        );
        // Create a successful response
        WriteResponse { error: 0 }
    }

    fn handle_get_version_request(&self, req: &GetVersionRequest) -> GetVersionResponse {
        // Lock the version history to find the requested version
        let version_history = self.version_history.lock().unwrap();

        if let Some((key, value)) = version_history.get(req.version as usize - 1) {
            // Create a successful response
            let response = GetVersionResponse {
                error: GetVersionResponseError::NoError as u8,
                key: key.clone().into_bytes(),
                value: value.clone().into_bytes(),
                version: req.version,
            };
            return response;
        }

        // If the version is not found, return an error response
        GetVersionResponse {
            error: GetVersionResponseError::KeyNotFound as u8,
            key: Vec::new(),   // No key in the error case
            value: Vec::new(), // No value in the error case
            version: req.version,
        }
    }

    fn handle_query_version_request(&self, _req: &QueryVersionRequest) -> QueryVersionResponse {
        // Lock the current version to read its value
        let current_version = self.current_version.lock().unwrap();

        println!("sent version: {}", *current_version);

        if *current_version > 0 {
            // Create the response with the latest version
            QueryVersionResponse {
                version: *current_version,
            }
        } else {
            // No data available
            QueryVersionResponse { version: 0 }
        }
    }

    /// Callback for handling new requests
    fn handle_announce_shard_request(&self, _req: &AnnounceShardRequest) -> AnnounceShardResponse {
        unimplemented!()
    }

    fn handle_get_client_shard_info_request(
        &self,
        _req: &GetClientShardInfoRequest,
    ) -> GetClientShardInfoResponse {
        unimplemented!()
    }

    fn handle_read_request(&self, _req: &ReadRequest) -> ReadResponse {
        unimplemented!()
    }

    fn handle_get_shared_peers_request(
        &self,
        _req: &GetSharedPeersRequest,
    ) -> GetSharedPeersResponse {
        unimplemented!()
    }

    /// Callbacks for handling responses to outbound requests
    fn handle_announce_shard_response(&self, _res: &AnnounceShardResponse) {
        // nothing to do
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

#[tokio::main]
async fn main() -> Result<()> {
    let write_shard_router = WriteShard::new();
    let mut write_shard_server = RouterBuilder::new(write_shard_router, None);
    let writer_ip_port = write_shard_server.bind().await?;

    let shard_id: u128 = rand::thread_rng().gen();

    let client1 = write_shard_server.get_router_client();
    tokio::spawn(async move {
        let mut interval = time::interval(time::Duration::from_secs(3));
        loop {
            interval.tick().await;

            let announce_request = AnnounceShardRequest {
                shard_type: ShardType::WriteShard,
                shard_id: shard_id,
                ip: writer_ip_port.ip().to_bits(),
                port: writer_ip_port.port(),
            };

            if let Err(e) = client1
                .queue_request::<AnnounceShardRequest>(announce_request, MAIN_INSTANCE_IP_PORT)
                .await
            {
                eprintln!("Failed to send AnnounceShardRequest: {:?}", e);
            }
        }
    });

    tokio::spawn(async move {
        if let Err(e) = write_shard_server.listen().await {
            eprintln!("Server failed: {:?}", e);
        }
    })
    .await?;

    Ok(())
}
