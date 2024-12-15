use std::sync::{Arc, Mutex};

use crate::io::router::{RouterBuilder, RouterClient, RouterHandler};
use crate::messages::{
    requests::{
        announce_shard_request::AnnounceShardRequest,
        get_client_shard_info_request::GetClientShardInfoRequest,
        get_shared_peers_request::GetSharedPeersRequest,
        query_version_request::QueryVersionRequest, read_request::ReadRequest,
        write_request::WriteRequest,
    },
    responses::{
        announce_shard_response::AnnounceShardResponse,
        get_client_shard_info_response::GetClientShardInfoResponse,
        get_shared_peers_response::GetSharedPeersResponse,
        query_version_response::QueryVersionResponse, read_response::ReadResponse,
        write_response::WriteResponse,
    },
};

/// A helper struct to use in testing. It stores any outbound responses so they can be asserted in unit tests
#[allow(unused)]
pub struct TestRouterClient {
    pub query_version_responses: Arc<Mutex<Vec<QueryVersionResponse>>>,
    pub announce_shard_responses: Arc<Mutex<Vec<AnnounceShardResponse>>>,
    pub get_client_shard_info_responses: Arc<Mutex<Vec<GetClientShardInfoResponse>>>,
    pub get_shared_peers_responses: Arc<Mutex<Vec<GetSharedPeersResponse>>>,
    pub read_responses: Arc<Mutex<Vec<ReadResponse>>>,
    pub write_responses: Arc<Mutex<Vec<WriteResponse>>>,

    router: RouterBuilder<TestRouterClientHandler>,
}

#[allow(unused)]
impl TestRouterClient {
    pub fn new() -> Self {
        let query_version_responses = Arc::new(Mutex::new(Vec::new()));
        let announce_shard_responses = Arc::new(Mutex::new(Vec::new()));
        let get_client_shard_info_responses = Arc::new(Mutex::new(Vec::new()));
        let get_shared_peers_responses = Arc::new(Mutex::new(Vec::new()));
        let read_responses = Arc::new(Mutex::new(Vec::new()));
        let write_responses = Arc::new(Mutex::new(Vec::new()));

        let router_handler = TestRouterClientHandler {
            query_version_responses: query_version_responses.clone(),
            announce_shard_responses: announce_shard_responses.clone(),
            get_client_shard_info_responses: get_client_shard_info_responses.clone(),
            get_shared_peers_responses: get_shared_peers_responses.clone(),
            read_responses: read_responses.clone(),
            write_responses: write_responses.clone(),
        };
        let router = RouterBuilder::new(router_handler, None);

        TestRouterClient {
            query_version_responses,
            announce_shard_responses,
            get_client_shard_info_responses,
            get_shared_peers_responses,
            read_responses,
            write_responses,
            router,
        }
    }

    pub fn get_client(&self) -> RouterClient<TestRouterClientHandler> {
        self.router.get_router_client()
    }
}

pub struct TestRouterClientHandler {
    query_version_responses: Arc<Mutex<Vec<QueryVersionResponse>>>,
    announce_shard_responses: Arc<Mutex<Vec<AnnounceShardResponse>>>,
    get_client_shard_info_responses: Arc<Mutex<Vec<GetClientShardInfoResponse>>>,
    get_shared_peers_responses: Arc<Mutex<Vec<GetSharedPeersResponse>>>,
    read_responses: Arc<Mutex<Vec<ReadResponse>>>,
    write_responses: Arc<Mutex<Vec<WriteResponse>>>,
}

impl RouterHandler for TestRouterClientHandler {
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

    fn handle_read_request(&self, _req: &ReadRequest) -> ReadResponse {
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

    fn handle_get_version_request(
        &self,
        _req: &crate::messages::requests::get_version_request::GetVersionRequest,
    ) -> crate::messages::responses::get_version_response::GetVersionResponse {
        unimplemented!()
    }

    fn handle_get_version_response(
        &self,
        _res: &crate::messages::responses::get_version_response::GetVersionResponse,
    ) {
        unimplemented!()
    }

    /// Callbacks for handling responses to outbound requests
    fn handle_announce_shard_response(&self, res: &AnnounceShardResponse) {
        let mut arr = self.announce_shard_responses.lock().unwrap();
        arr.push(res.clone());
    }

    fn handle_get_client_shard_info_response(&self, res: &GetClientShardInfoResponse) {
        let mut arr = self.get_client_shard_info_responses.lock().unwrap();
        arr.push(res.clone());
    }

    fn handle_query_version_response(&self, res: &QueryVersionResponse) {
        let mut arr = self.query_version_responses.lock().unwrap();
        arr.push(res.clone());
    }

    fn handle_read_response(&self, res: &ReadResponse) {
        let mut arr = self.read_responses.lock().unwrap();
        arr.push(res.clone());
    }

    fn handle_write_response(&self, res: &WriteResponse) {
        let mut arr = self.write_responses.lock().unwrap();
        arr.push(res.clone());
    }

    fn handle_get_shared_peers_response(&self, res: &GetSharedPeersResponse) {
        let mut arr = self.get_shared_peers_responses.lock().unwrap();
        arr.push(res.clone());
    }
}
