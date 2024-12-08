pub mod io;
pub mod messages;

use crate::io::router::{RouterBuilder, RouterHandler};

use messages::requests::announce_shard_request::AnnounceShardRequest;
use messages::requests::get_client_shard_info_request::GetClientShardInfoRequest;
use messages::requests::get_shared_peers_request::GetSharedPeersRequest;
use messages::requests::query_version_request::QueryVersionRequest;
use messages::requests::read_request::ReadRequest;
use messages::requests::write_request::WriteRequest;

use messages::responses::announce_shard_response::AnnounceShardResponse;
use messages::responses::get_client_shard_info_response::GetClientShardInfoResponse;
use messages::responses::get_shared_peers_response::GetSharedPeersResponse;
use messages::responses::query_version_response::QueryVersionResponse;
use messages::responses::read_response::ReadResponse;
use messages::responses::write_response::WriteResponse;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use anyhow::{Result};

struct InfoRouter {
}

impl InfoRouter {
    pub fn new() -> Self {
        InfoRouter {

        }
    }
}

impl RouterHandler for InfoRouter {
    /// Callback for handling new requests
    fn handle_announce_shard_request(&self, req: &AnnounceShardRequest) -> AnnounceShardResponse {
        unimplemented!()
    }
    fn handle_announce_shard_response(&self, res: &AnnounceShardResponse) {
        unimplemented!()
    }

    fn handle_get_client_shard_info_request(&self, req: &GetClientShardInfoRequest) -> GetClientShardInfoResponse {
        unimplemented!()
    }

    /// Info server does not handle requests in relation to the actual key/value state
    /// we could change this to return an error response in the future
    fn handle_query_version_request(&self, req: &QueryVersionRequest) -> QueryVersionResponse {
        unimplemented!()
    }
    fn handle_read_request(&self, req: &ReadRequest) -> ReadResponse {unimplemented!()}
    fn handle_write_request(&self, req: &WriteRequest) -> WriteResponse {unimplemented!()}

    fn handle_get_shared_peers_request(&self, req: &GetSharedPeersRequest) -> GetSharedPeersResponse {
        unimplemented!()
    }

    /// Callbacks for handling responses to outbound requests
    

    fn handle_get_client_shard_info_response(&self, res: &GetClientShardInfoResponse) {
        unimplemented!()
    }

    fn handle_query_version_response(&self, res: &QueryVersionResponse) {
        unimplemented!()
    }

    fn handle_read_response(&self, res: &ReadResponse) {
    }

    fn handle_write_response(&self, res: &WriteResponse) {
        unimplemented!()
    }

    fn handle_get_shared_peers_response(&self, res: &GetSharedPeersResponse) {
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
    }).await?
}
