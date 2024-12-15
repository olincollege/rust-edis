use crate::io::router::{RouterBuilder, RouterHandler};
use anyhow::{Ok, Result};
use messages::requests::announce_shard_request::ShardType;
use messages::requests::get_client_shard_info_request::GetClientShardInfoRequest;
use messages::requests::write_request::WriteRequest;
use messages::responses::get_client_shard_info_response::GetClientShardInfoResponse;
use messages::responses::write_response::WriteResponse;
use rand::Rng;
use std::collections::HashMap;
use std::net::{Ipv6Addr, SocketAddrV6};
use std::sync::{Arc, Mutex};
use tokio::time;
mod integration;
mod io;
mod messages;
mod utils;
use crate::messages::{
    requests::{
        announce_shard_request::{AnnounceMessageType, AnnounceShardRequest},
        get_shared_peers_request::GetSharedPeersRequest,
        get_version_request::GetVersionRequest,
        query_version_request::QueryVersionRequest,
        read_request::ReadRequest,
    },
    responses::{
        announce_shard_response::AnnounceShardResponse,
        get_shared_peers_response::GetSharedPeersResponse,
        get_version_response::GetVersionResponse, query_version_response::QueryVersionResponse,
        read_response::ReadResponse,
    },
};
use crate::utils::constants::MAIN_INSTANCE_IP_PORT;

#[derive(Clone, Debug)]
pub struct ReadShard {
    writer_id: Arc<Mutex<u16>>,
    peers: Arc<Mutex<Vec<SocketAddrV6>>>,
    requested_version: Arc<Mutex<u64>>,
    current_version: Arc<Mutex<u64>>,
    history: Arc<Mutex<Vec<(String, String)>>>,
    data: Arc<Mutex<HashMap<String, String>>>,
}

impl RouterHandler for ReadShard {
    fn handle_announce_shard_response(&self, res: &AnnounceShardResponse) {
        println!("handing announce shard response (reader)");
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
        println!("handing get shared peers response (reader)");
        let mut peers = self.peers.lock().unwrap();
        *peers = res
            .peer_ips
            .iter()
            .map(|x| SocketAddrV6::new(Ipv6Addr::from(x.0), x.1, 0, 0))
            .collect();
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
        QueryVersionResponse {
            version: *self.current_version.lock().unwrap(),
        }
    }

    fn handle_get_version_request(&self, req: &GetVersionRequest) -> GetVersionResponse {
        let res = self.history.lock().unwrap();
        if req.version <= *self.current_version.lock().unwrap() {
            GetVersionResponse {
                error: 0,
                key: (res[req.version as usize - 1].0).as_bytes().to_vec(),
                value: (res[req.version as usize - 1].1).as_bytes().to_vec(),
                version: req.version,
            }
        } else {
            GetVersionResponse {
                error: 1,
                key: Vec::new(),
                value: Vec::new(),
                version: req.version,
            }
        }
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
    pub fn new() -> ReadShard {
        ReadShard {
            writer_id: Arc::new(Mutex::new(0)),
            peers: Arc::new(Mutex::new(Vec::new())),
            requested_version: Arc::new(Mutex::new(0)),
            current_version: Arc::new(Mutex::new(0)),
            history: Arc::new(Mutex::new(Vec::new())),
            data: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let read_shard_router = ReadShard::new();
    let mut read_shard_server = RouterBuilder::new(read_shard_router, None);
    let read_shard_router = read_shard_server.get_handler_arc();
    let reader_ip_port = read_shard_server.bind().await?;

    println!("hi from read shard!");

    /*/
    let client0 = read_shard_server.get_router_client();
    tokio::spawn(async move {
        println!("sending announce shard request from read");
        let announce_request = AnnounceShardRequest {
            shard_type: ShardType::ReadShard,
            message_type: AnnounceMessageType::NewAnnounce as u8,
            ip: reader_ip_port.ip().to_bits(),
            port: reader_ip_port.port(),
        };
        if let Err(e) = client0
            .queue_request::<AnnounceShardRequest>(announce_request, MAIN_INSTANCE_IP_PORT)
            .await
        {
            eprintln!("Failed to send AnnounceShardRequest: {:?}", e);
        }
    });
    */

    let client1 = read_shard_server.get_router_client();
    tokio::spawn(async move {
        println!("hi from read shard loop");
        let mut interval = time::interval(time::Duration::from_secs(3));
        loop {
            interval.tick().await;

            let announce_request = AnnounceShardRequest {
                shard_type: ShardType::ReadShard,
                message_type: AnnounceMessageType::ReAnnounce as u8,
                ip: reader_ip_port.ip().to_bits(),
                port: reader_ip_port.port(),
            };

            println!("sending announce shard request from read");
            if let Err(e) = client1
                .queue_request::<AnnounceShardRequest>(announce_request, MAIN_INSTANCE_IP_PORT)
                .await
            {
                eprintln!("Failed to send AnnounceShardRequest: {:?}", e);
            }
        }
    });

    /*
    let router_clone_2 = read_shard_router.clone();
    let client2 = read_shard_server.get_router_client();
    tokio::spawn({
        async move {
            let mut interval = time::interval(time::Duration::from_secs(3));
            loop {
                interval.tick().await;

                println!("sending shared peers request (reader)");

                let get_peers_request = GetSharedPeersRequest {
                    writer_number: router_clone_2.writer_id.lock().unwrap().clone(),
                };

                if let Err(e) = client2
                    .queue_request::<GetSharedPeersRequest>(
                        get_peers_request,
                        MAIN_INSTANCE_IP_PORT,
                    )
                    .await
                {
                    eprintln!("Failed to send GetSharedPeersRequest: {:?}", e);
                }
            }
        }
    });
    */

    /*
    let client3 = read_shard_server.get_router_client();
    let client4 = read_shard_server.get_router_client();
    let router_clone_3 = read_shard_router.clone();
    tokio::spawn({
        async move {
            let mut interval = time::interval(time::Duration::from_secs(3));
            loop {
                interval.tick().await;

                let peer_ip_port = {
                    let peers = router_clone_3.peers.lock().unwrap();
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
                    .queue_request::<QueryVersionRequest>(query_version_request, peer_ip_port)
                    .await
                {
                    eprintln!("Failed to send QueryVersionRequest: {:?}", e);
                }

                let (current_version, requested_version) = {
                    let curr_ver = router_clone_3.current_version.lock().unwrap().clone();
                    let req_ver = router_clone_3.requested_version.lock().unwrap().clone();
                    (curr_ver, req_ver)
                };

                if requested_version > current_version {
                    let get_version_request = GetVersionRequest {
                        version: current_version + 1,
                    };

                    if let Err(e) = client4
                        .queue_request::<GetVersionRequest>(get_version_request, peer_ip_port)
                        .await
                    {
                        eprintln!("Failed to send GetVersionRequest: {:?}", e);
                    }
                }
            }
        }
    });
    */

    tokio::spawn(async move {
        println!("hi from read shard listen block");
        if let Err(e) = read_shard_server.listen().await {
            eprintln!("Server failed: {:?}", e);
        }
    })
    .await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_handle_read_request() {
        let read_shard = ReadShard::new();

        read_shard
            .data
            .lock()
            .unwrap()
            .insert("key1".to_string(), "value1".to_string());

        let read_request = ReadRequest {
            key: "key1".to_string().into_bytes(),
        };

        let response = read_shard.handle_read_request(&read_request);

        assert_eq!(response.error, 0);
        assert_eq!(response.value, "value1".to_string().into_bytes());
    }

    #[test]
    fn test_handle_announce_shard_response() {
        let read_shard = ReadShard::new();

        let announce_shard_response = AnnounceShardResponse { writer_number: 1 };

        read_shard.handle_announce_shard_response(&announce_shard_response);

        let writer_id = read_shard.writer_id.lock().unwrap();
        assert_eq!(*writer_id, 1);
    }

    #[test]
    fn test_handle_get_shared_peers_response() {
        let read_shard = ReadShard::new();

        let res = GetSharedPeersResponse {
            peer_ips: vec![(1, 8084)],
        };

        read_shard.handle_get_shared_peers_response(&res);

        let peers = read_shard.peers.lock().unwrap();
        assert_eq!(
            *peers,
            vec![SocketAddrV6::new(
                Ipv6Addr::from([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1]),
                8084,
                0,
                0
            )]
        );
    }

    #[test]
    fn test_handle_query_version_response() {
        let read_shard = ReadShard::new();

        let res = QueryVersionResponse { version: 1 };

        read_shard.handle_query_version_response(&res);

        let requested_version = read_shard.requested_version.lock().unwrap();
        assert_eq!(*requested_version, 1);
    }

    #[test]
    fn test_handle_get_version_response() {
        let read_shard = ReadShard::new();

        let get_version_response = GetVersionResponse {
            key: b"key".to_vec(),
            value: b"value".to_vec(),
            error: 0,
            version: 1,
        };

        read_shard.handle_get_version_response(&get_version_response);

        let current_version = read_shard.current_version.lock().unwrap();
        assert_eq!(*current_version, 1);
    }

    #[test]
    fn test_handle_query_version_request() {
        let read_shard = ReadShard::new();

        let req = QueryVersionRequest {};

        let res = read_shard.handle_query_version_request(&req);

        assert_eq!(res.version, 0);
    }

    #[test]
    fn test_handle_get_version_request() {
        let read_shard = ReadShard::new();

        read_shard
            .data
            .lock()
            .unwrap()
            .insert("key".to_string(), "value".to_string());
        read_shard
            .history
            .lock()
            .unwrap()
            .push(("key".to_string(), "value".to_string()));

        *read_shard.current_version.lock().unwrap() = 1;

        let req = GetVersionRequest { version: 1 };

        let res = read_shard.handle_get_version_request(&req);

        assert_eq!(res.error, 0);
        assert_eq!(res.key, b"key".to_vec());
        assert_eq!(res.value, b"value".to_vec());
        assert_eq!(res.version, 1);
    }
}
