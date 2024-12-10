use crate::io::router::{RouterBuilder, RouterHandler};
use anyhow::{Ok, Result};
use messages::requests::get_client_shard_info_request::GetClientShardInfoRequest;
use messages::requests::write_request::WriteRequest;
use messages::responses::get_client_shard_info_response::GetClientShardInfoResponse;
use messages::responses::write_response::WriteResponse;
use rand::Rng;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
mod io;
mod messages;
use crate::messages::{
    requests::{
        announce_shard_request::AnnounceShardRequest,
        get_shared_peers_request::GetSharedPeersRequest, get_version_request::GetVersionRequest,
        query_version_request::QueryVersionRequest, read_request::ReadRequest,
    },
    responses::{
        announce_shard_response::AnnounceShardResponse,
        get_shared_peers_response::GetSharedPeersResponse,
        get_version_response::GetVersionResponse, query_version_response::QueryVersionResponse,
        read_response::ReadResponse,
    },
};

static MAIN_INSTANCE_IP_PORT: ([u8; 16], u16) =
    ([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1], 8080);
#[derive(Clone)]
pub struct ReadShard {
    _reader_ip_port: Arc<Mutex<([u8; 16], u16)>>,
    writer_id: Arc<Mutex<u16>>,
    writer_ip_port: Arc<Mutex<([u8; 16], u16)>>,
    peers: Arc<Mutex<Vec<([u8; 16], u16)>>>,
    requested_version: Arc<Mutex<u64>>,
    current_version: Arc<Mutex<u64>>,
    history: Arc<Mutex<Vec<(String, String)>>>,
    data: Arc<Mutex<HashMap<String, String>>>,
}

impl RouterHandler for ReadShard {
    fn handle_announce_shard_response(&self, res: &AnnounceShardResponse) {
        let writer_number = res.writer_number;
        let mut writer_id = self.writer_id.lock().unwrap();
        *writer_id = writer_number;
    }

    fn handle_read_request(&self, req: &ReadRequest) -> ReadResponse {
        let key = String::from_utf8_lossy(&req.key).into_owned();
        let value = self.data.lock().unwrap().get(&key).cloned();
        if value.is_none() {
            return ReadResponse {
                error: 1,
                key: req.key.clone(),
                value: Vec::new(),
            };
        } else {
            return ReadResponse {
                error: 0,
                key: req.key.clone(),
                value: value.unwrap().into_bytes(),
            };
        }
    }

    fn handle_get_shared_peers_response(&self, res: &GetSharedPeersResponse) {
        let mut peers = self.peers.lock().unwrap();
        *peers = res.peer_ips.clone();
        let mut writer_ip_port = self.writer_ip_port.lock().unwrap();
        *writer_ip_port = res.peer_ips[0];
    }

    fn handle_query_version_response(&self, res: &QueryVersionResponse) {
        let mut requested_version = self.requested_version.lock().unwrap();
        *requested_version = res.version;
    }

    fn handle_get_version_response(&self, res: &GetVersionResponse) {
        let mut current_version = self.current_version.lock().unwrap();

        if res.error == 0 {
            if res.version == *current_version + 1 {
                *current_version = res.version;
                let mut history = self.history.lock().unwrap();
                let mut data = self.data.lock().unwrap();
                data.insert(
                    String::from_utf8_lossy(&res.key).into_owned(),
                    String::from_utf8_lossy(&res.value).into_owned(),
                );
                history.push((
                    String::from_utf8_lossy(&res.key).into_owned(),
                    String::from_utf8_lossy(&res.value).into_owned(),
                ));

                let mut requested_version = self.requested_version.lock().unwrap();
                *requested_version = *current_version;
            }
        }
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

    fn handle_query_version_request(&self, _req: &QueryVersionRequest) -> QueryVersionResponse {
        unimplemented!()
    }

    fn handle_get_version_request(&self, _req: &GetVersionRequest) -> GetVersionResponse {
        unimplemented!()
    }

    fn handle_write_request(&self, _req: &WriteRequest) -> WriteResponse {
        unimplemented!()
    }

    fn handle_get_shared_peers_request(
        &self,
        _req: &GetSharedPeersRequest,
    ) -> GetSharedPeersResponse {
        unimplemented!()
    }

    fn handle_get_client_shard_info_response(&self, _res: &GetClientShardInfoResponse) {
        unimplemented!()
    }

    fn handle_read_response(&self, _res: &ReadResponse) {
        unimplemented!()
    }

    fn handle_write_response(&self, _res: &WriteResponse) {
        unimplemented!()
    }
}

impl ReadShard {
    pub fn new(reader_ip_port: Arc<Mutex<([u8; 16], u16)>>) -> ReadShard {
        ReadShard {
            _reader_ip_port: reader_ip_port,
            writer_id: Arc::new(Mutex::new(0)),
            writer_ip_port: Arc::new(Mutex::new(([0; 16], 0))),
            peers: Arc::new(Mutex::new(Vec::new())),
            requested_version: Arc::new(Mutex::new(0)),
            current_version: Arc::new(Mutex::new(0)),
            history: Arc::new(Mutex::new(Vec::new())),
            data: Arc::new(Mutex::new(HashMap::new())),
        }
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
    let reader_ip_port = ([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1], 8084);

    let info_router = ReadShard::new(Arc::new(Mutex::new(reader_ip_port)));
    let info_router_1 = info_router.clone();
    let info_server = RouterBuilder::new(info_router_1, None);

    let client1 = info_server.get_router_client();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(3));
        loop {
            interval.tick().await;

            let announce_request = AnnounceShardRequest {
                shard_type: 1,
                ip: reader_ip_port.0,
                port: reader_ip_port.1,
            };

            let _ = client1
                .queue_request::<AnnounceShardRequest>(
                    announce_request,
                    socket_addr_to_string(MAIN_INSTANCE_IP_PORT),
                )
                .await;
        }
    });

    let info_router_2 = info_router.clone();
    let client2 = info_server.get_router_client();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(3));
        loop {
            interval.tick().await;

            let get_peers_request = GetSharedPeersRequest {
                writer_number: info_router_2.writer_id.lock().unwrap().clone(),
            };
            let _ = client2
                .queue_request::<GetSharedPeersRequest>(
                    get_peers_request,
                    socket_addr_to_string(MAIN_INSTANCE_IP_PORT),
                )
                .await;
        }
    });

    let client3 = info_server.get_router_client();
    let client4 = info_server.get_router_client();

    let info_router_3 = info_router.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(3));
        loop {
            interval.tick().await;

            let peer_ip_port = {
                let peers = info_router_3.peers.lock().unwrap();
                if peers.is_empty() {
                    eprintln!("No peers available for query.");
                    continue;
                }
                let mut rng = rand::thread_rng();
                let index = rng.gen_range(0..peers.len());
                peers[index]
            };

            let query_version_request = QueryVersionRequest {};
            if let Err(e) = client3
                .queue_request::<QueryVersionRequest>(
                    query_version_request,
                    socket_addr_to_string(peer_ip_port),
                )
                .await
            {
                eprintln!("Failed to send QueryVersionRequest: {:?}", e);
            }

            let (current_version, requested_version) = {
                let curr_ver = info_router_3.current_version.lock().unwrap().clone();
                let req_ver = info_router_3.requested_version.lock().unwrap().clone();
                (curr_ver, req_ver)
            };

            if requested_version > current_version {
                let get_version_request = GetVersionRequest {
                    version: current_version + 1,
                };

                if let Err(e) = client4
                    .queue_request::<GetVersionRequest>(
                        get_version_request,
                        socket_addr_to_string(peer_ip_port),
                    )
                    .await
                {
                    eprintln!("Failed to send GetVersionRequest: {:?}", e);
                }
            }
        }
    });

    tokio::spawn(async move {
        info_server.listen().await.unwrap();
        Ok(())
    });

    Ok(())
}
