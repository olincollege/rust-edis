pub mod integration;
pub mod io;
pub mod messages;
pub mod utils;

use std::mem::uninitialized;

use crate::io::router::{RouterBuilder, RouterHandler};

use clap::{Args, Parser};
use messages::requests::announce_shard_request::{AnnounceShardRequest, ShardType};
use messages::requests::get_client_shard_info_request::GetClientShardInfoRequest;
use messages::requests::get_shared_peers_request::GetSharedPeersRequest;
use messages::requests::get_version_request::GetVersionRequest;
use messages::requests::query_version_request::QueryVersionRequest;

use messages::requests::read_request::ReadRequest;
use messages::requests::write_request::WriteRequest;

use messages::responses::announce_shard_response::AnnounceShardResponse;
use messages::responses::get_client_shard_info_response::GetClientShardInfoResponse;
use messages::responses::get_shared_peers_response::GetSharedPeersResponse;
use messages::responses::get_version_response::GetVersionResponse;

use messages::responses::query_version_response::QueryVersionResponse;
use messages::responses::read_response::ReadResponse;
use messages::responses::write_response::WriteResponse;

use anyhow::Result;
use rand::seq::SliceRandom;
use std::sync::{Arc, Mutex};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use utils::constants::MAIN_INSTANCE_IP_PORT;

#[derive(Clone)]
struct AnnounceInfo {
    ip: u128,
    port: u16,
    announce_id: u128,
}

#[derive(Clone)]
struct ReaderWriterBlock {
    writer: Option<AnnounceInfo>,
    readers: Vec<AnnounceInfo>,
}

struct InfoRouter {
    reader_writers: Arc<Mutex<Vec<ReaderWriterBlock>>>,
}

impl InfoRouter {
    pub fn new(num_writers: u16) -> Self {
        InfoRouter {
            reader_writers: Arc::new(Mutex::new(vec![
                ReaderWriterBlock {
                    writer: None,
                    readers: Vec::new()
                };
                num_writers as usize
            ])),
        }
    }
}

impl RouterHandler for InfoRouter {
    /// Callback for handling new requests
    fn handle_announce_shard_request(&self, req: &AnnounceShardRequest) -> AnnounceShardResponse {
        let mut reader_writers = self.reader_writers.lock().unwrap();

        // check if message is reannounce of already announced shard
        for block in reader_writers.iter_mut().enumerate() {
            if let Some(writer) = &mut block.1.writer {
                if writer.announce_id == req.shard_id {
                    // update ip/port
                    writer.ip = req.ip;
                    writer.port = req.port;
                    // already announced
                    return AnnounceShardResponse {
                        writer_number: block.0 as u16,
                    };
                }
            }
            for reader in block.1.readers.iter_mut() {
                if reader.announce_id == req.shard_id {
                    // update ip/port
                    reader.ip = req.ip;
                    reader.port = req.port;
                    // already announced
                    return AnnounceShardResponse {
                        writer_number: block.0 as u16,
                    };
                }
            }
        }

        println!("handling announce shard request");
        match req.shard_type {
            ShardType::ReadShard => {
                // find the writer with the smallest number of readers and attach there
                let writer_idx = reader_writers
                    .iter()
                    .enumerate()
                    .min_by(|(_, a), (_, b)| a.readers.len().cmp(&b.readers.len()))
                    .unwrap()
                    .0;
                reader_writers[writer_idx].readers.push(AnnounceInfo {
                    ip: req.ip,
                    port: req.port,
                    announce_id: req.shard_id,
                });

                AnnounceShardResponse {
                    writer_number: writer_idx as u16,
                }
            }
            ShardType::WriteShard => {
                let mut reader_writers = self.reader_writers.lock().unwrap();
                let first_empty_idx = reader_writers
                    .iter()
                    .position(|block| block.writer.is_none());
                match first_empty_idx {
                    None => {
                        println!("too many write shards already attached, skipping");
                        // todo: have this support an error code
                        AnnounceShardResponse { writer_number: 0 }
                    }
                    Some(idx) => {
                        reader_writers[idx].writer = Some(AnnounceInfo {
                            ip: req.ip,
                            port: req.port,
                            announce_id: req.shard_id,
                        });
                        AnnounceShardResponse {
                            writer_number: idx as u16,
                        }
                    }
                }
            }
        }
    }

    fn handle_get_client_shard_info_request(
        &self,
        _req: &GetClientShardInfoRequest,
    ) -> GetClientShardInfoResponse {
        let reader_writers = self.reader_writers.lock().unwrap();

        let mut writers: Vec<(u128, u16)> = Vec::new();
        let mut readers: Vec<(u128, u16)> = Vec::new();

        for writer_block in reader_writers.iter() {
            match &writer_block.writer {
                Some(writer) => {
                    writers.push((writer.ip, writer.port));
                    let reader = writer_block.readers.choose(&mut rand::thread_rng());
                    match reader {
                        Some(reader) => {
                            readers.push((reader.ip, reader.port));
                        }
                        None => {
                            // error
                            println!("(error) on client shard info req 1");
                            return GetClientShardInfoResponse {
                                num_write_shards: 0,
                                read_shard_info: Vec::new(),
                                write_shard_info: Vec::new(),
                            };
                        }
                    }
                }
                None => {
                    // error
                    println!("(error) on client shard info req 2");
                    return GetClientShardInfoResponse {
                        num_write_shards: 0,
                        read_shard_info: Vec::new(),
                        write_shard_info: Vec::new(),
                    };
                }
            }
        }
        GetClientShardInfoResponse {
            num_write_shards: writers.len() as u16,
            write_shard_info: writers,
            read_shard_info: readers,
        }
    }

    fn handle_get_shared_peers_request(
        &self,
        req: &GetSharedPeersRequest,
    ) -> GetSharedPeersResponse {
        let mut reader_writers = self.reader_writers.lock().unwrap();

        let mut peer_ips: Vec<(u128, u16)> = Vec::new();

        if (req.writer_number as usize) < reader_writers.len() {
            let writer_block = &mut reader_writers[req.writer_number as usize];
            match &writer_block.writer {
                Some(writer) => {
                    peer_ips.push((writer.ip, writer.port));
                    for reader in &writer_block.readers {
                        peer_ips.push((reader.ip, reader.port));
                    }
                }
                None => {}
            }
        }

        GetSharedPeersResponse { peer_ips: peer_ips }
    }

    // Unused requests
    fn handle_query_version_request(&self, req: &QueryVersionRequest) -> QueryVersionResponse {
        unimplemented!()
    }
    fn handle_read_request(&self, req: &ReadRequest) -> ReadResponse {
        unimplemented!()
    }
    fn handle_write_request(&self, req: &WriteRequest) -> WriteResponse {
        unimplemented!()
    }
    fn handle_get_version_request(&self, req: &GetVersionRequest) -> GetVersionResponse {
        unimplemented!();
    }

    // Unused responses
    fn handle_announce_shard_response(&self, res: &AnnounceShardResponse) {
        unimplemented!()
    }
    fn handle_get_client_shard_info_response(&self, res: &GetClientShardInfoResponse) {
        unimplemented!()
    }
    fn handle_query_version_response(&self, res: &QueryVersionResponse) {
        unimplemented!()
    }
    fn handle_read_response(&self, res: &ReadResponse) {}
    fn handle_write_response(&self, res: &WriteResponse) {
        unimplemented!()
    }
    fn handle_get_shared_peers_response(&self, res: &GetSharedPeersResponse) {
        unimplemented!()
    }
    fn handle_get_version_response(&self, res: &GetVersionResponse) {
        unimplemented!()
    }
}

#[derive(Parser, Debug)]
pub struct InfoArgs {
    #[arg(long, default_value_t = 4)]
    write_shards: u16,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = InfoArgs::parse();
    let info_router = InfoRouter::new(args.write_shards);
    let mut info_server = RouterBuilder::new(info_router, Some(MAIN_INSTANCE_IP_PORT));
    tokio::spawn(async move {
        info_server.bind().await?;
        info_server.listen().await?;
        Ok(())
    })
    .await?
}

#[cfg(test)]
mod tests {
    use rand::Rng;
    use rust_edis::integration::test_setup;
    use serial_test::serial;
    use utils::test_client::{self, TestRouterClient};

    use super::*;
    use std::net::{Ipv6Addr, SocketAddrV6};

    #[tokio::test]
    #[serial]
    async fn test_shard_attachment() {
        test_setup::setup_test();

        let test_router_client = TestRouterClient::new();
        let test_client = test_router_client.get_client();

        let write_shards = 2;
        let read_shards = 4;

        let local = SocketAddrV6::new(Ipv6Addr::LOCALHOST, 8080, 0, 0);
        let mut info_router = RouterBuilder::new(InfoRouter::new(2), Some(local));

        tokio::spawn(async move {
            info_router.bind().await;
            info_router.listen().await;
        });
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

        // send a bunch of announcements
        for i in 0..write_shards {
            test_client
                .queue_request(
                    AnnounceShardRequest {
                        shard_type: ShardType::WriteShard,
                        shard_id: rand::thread_rng().gen(),
                        ip: i,
                        port: i as u16,
                    },
                    local,
                )
                .await
                .unwrap();

            for j in 0..read_shards {
                test_client
                    .queue_request(
                        AnnounceShardRequest {
                            shard_type: ShardType::ReadShard,
                            shard_id: rand::thread_rng().gen(),
                            ip: (j + 1) * 100,
                            port: ((j + 1) * 100) as u16,
                        },
                        local,
                    )
                    .await
                    .unwrap();
            }
        }

        // test peer lists
        for i in 0..write_shards {
            test_client
                .queue_request(
                    GetSharedPeersRequest {
                        writer_number: i as u16,
                    },
                    local,
                )
                .await
                .unwrap();
        }

        // test client peer lists
        let num_get_client_shard_info_requests = 2;
        for _ in 0..num_get_client_shard_info_requests {
            test_client
                .queue_request(GetClientShardInfoRequest {}, local)
                .await
                .unwrap();
        }

        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

        // assert responses
        let client_shard_info_responses = test_router_client
            .get_client_shard_info_responses
            .lock()
            .unwrap();
        let shared_peers_responses = test_router_client
            .get_shared_peers_responses
            .lock()
            .unwrap();

        assert_eq!(shared_peers_responses.len(), write_shards as usize);
        for i in 0..(write_shards as usize) {
            assert_eq!(shared_peers_responses[i].peer_ips[0].0, i as u128);
            assert_eq!(shared_peers_responses[i].peer_ips[0].1, i as u16);
            for j in 0..(read_shards as usize) {
                assert!(shared_peers_responses[i].peer_ips[1 + j].0 >= 100 as u128);
                assert!(shared_peers_responses[i].peer_ips[1 + j].1 >= 100 as u16);
            }
        }

        assert_eq!(
            client_shard_info_responses.len(),
            num_get_client_shard_info_requests as usize
        );
        assert_eq!(
            client_shard_info_responses[0].num_write_shards,
            write_shards as u16
        );
        assert_eq!(
            client_shard_info_responses[0].write_shard_info.len(),
            write_shards as usize
        );
        for i in 0..(write_shards as usize) {
            assert_eq!(
                client_shard_info_responses[0].write_shard_info[i].0,
                i as u128
            );
            assert_eq!(
                client_shard_info_responses[0].write_shard_info[i].1,
                i as u16
            );
        }
        assert_eq!(
            client_shard_info_responses[0].read_shard_info.len(),
            write_shards as usize
        );
    }
}
