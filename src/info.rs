pub mod io;
pub mod messages;

use std::mem::uninitialized;

use crate::io::router::{RouterBuilder, RouterHandler};

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
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
struct ReaderWriterBlock {
    writer: Option<(u128, u16)>,
    readers: Vec<(u128, u16)>
}

struct InfoRouter {
    num_writers: Arc<Mutex<u16>>,
    reader_writers: Arc<Mutex<Vec<ReaderWriterBlock>>>
}

impl InfoRouter {
    pub fn new(num_writers: u16) -> Self {
        InfoRouter {
            num_writers: Arc::new(Mutex::new(num_writers)),
            reader_writers: Arc::new(Mutex::new(vec![ReaderWriterBlock { writer: None, readers: Vec::new() }; num_writers as usize]))
        }
    }
}

impl RouterHandler for InfoRouter {
    /// Callback for handling new requests
    fn handle_announce_shard_request(&self, req: &AnnounceShardRequest) -> AnnounceShardResponse {
        match req.shard_type {
            ShardType::ReadShard => {
                let mut reader_writers = self.reader_writers.lock().unwrap();

                // find the writer with the smallest number of readers and attach there
                let writer_idx = reader_writers.iter().enumerate().min_by(|(_, a), (_, b)| {
                    a.readers.len().cmp(&b.readers.len())
                }).unwrap().0;
                reader_writers[writer_idx].readers.push((req.ip,req.port));

                AnnounceShardResponse {
                    writer_number: writer_idx as u16
                }
            }
            ShardType::WriteShard => {
                let mut reader_writers = self.reader_writers.lock().unwrap();
                let first_empty_idx = reader_writers.iter().position(|block| block.writer.is_none());
                match first_empty_idx {
                    None => {
                        println!("too many write shards already attached, skipping");
                        // todo: have this support an error code
                        AnnounceShardResponse {
                            writer_number: 0
                        }
                    } 
                    Some(idx) => {
                        reader_writers[idx].writer = Some((req.ip,req.port));
                        AnnounceShardResponse {
                            writer_number: idx as u16
                        }
                    }
                }
          }
        }
    }
    fn handle_announce_shard_response(&self, res: &AnnounceShardResponse) {
        unimplemented!()
    }

    fn handle_get_client_shard_info_request(
        &self,
        req: &GetClientShardInfoRequest,
    ) -> GetClientShardInfoResponse {
        unimplemented!()
    }

    fn handle_get_version_request(&self, req: &GetVersionRequest) -> GetVersionResponse {
        unimplemented!();
    }

    /// Info server does not handle requests in relation to the actual key/value state
    /// we could change this to return an error response in the future
    fn handle_query_version_request(&self, req: &QueryVersionRequest) -> QueryVersionResponse {
        unimplemented!()
    }
    fn handle_read_request(&self, req: &ReadRequest) -> ReadResponse {
        unimplemented!()
    }
    fn handle_write_request(&self, req: &WriteRequest) -> WriteResponse {
        unimplemented!()
    }

    fn handle_get_shared_peers_request(
        &self,
        req: &GetSharedPeersRequest,
    ) -> GetSharedPeersResponse {
        unimplemented!()
    }

    /// Callbacks for handling responses to outbound requests

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

#[tokio::main]
async fn main() -> Result<()> {
    let info_router = InfoRouter::new();
    let info_server = RouterBuilder::new(info_router, None);
    tokio::spawn(async move {
        info_server.listen().await?;
        Ok(())
    })
    .await?
}
