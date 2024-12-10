use std::sync::{Arc, RwLock};

use crate::io::router::{RouterBuilder, RouterHandler};
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
struct ExampleRouterHandler {
    debug_out: Arc<RwLock<Vec<Vec<u8>>>>,
}
impl RouterHandler for ExampleRouterHandler {
    /// Callback for handling new requests
    fn handle_announce_shard_request(&self, req: &AnnounceShardRequest) -> AnnounceShardResponse {
        unimplemented!()
    }

    fn handle_get_client_shard_info_request(
        &self,
        req: &GetClientShardInfoRequest,
    ) -> GetClientShardInfoResponse {
        unimplemented!()
    }

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
    fn handle_announce_shard_response(&self, res: &AnnounceShardResponse) {
        unimplemented!()
    }

    fn handle_get_client_shard_info_response(&self, res: &GetClientShardInfoResponse) {
        unimplemented!()
    }

    fn handle_query_version_response(&self, res: &QueryVersionResponse) {
        unimplemented!()
    }

    fn handle_read_response(&self, res: &ReadResponse) {
        self.debug_out.write().unwrap().push(res.value.clone());
    }

    fn handle_write_response(&self, res: &WriteResponse) {
        unimplemented!()
    }

    fn handle_get_shared_peers_response(&self, res: &GetSharedPeersResponse) {
        unimplemented!()
    }

    fn handle_get_version_request(
        &self,
        req: &crate::messages::requests::get_version_request::GetVersionRequest,
    ) -> crate::messages::responses::get_version_response::GetVersionResponse {
        unimplemented!()
    }

    fn handle_get_version_response(
        &self,
        res: &crate::messages::responses::get_version_response::GetVersionResponse,
    ) {
        unimplemented!()
    }
}

mod test {
    use std::sync::{Arc, RwLock};

    use crate::{
        io::{router::RouterBuilder, router_example::ExampleRouterHandler},
        messages::requests::{
            query_version_request::QueryVersionRequest, read_request::ReadRequest,
        },
    };
    use anyhow::{Ok, Result};

    #[tokio::test]
    async fn test_example_router() -> Result<()> {
        let debug_out1: Arc<RwLock<Vec<Vec<u8>>>> = Arc::new(RwLock::new(Vec::new()));
        let debug_out2: Arc<RwLock<Vec<Vec<u8>>>> = Arc::new(RwLock::new(Vec::new()));

        let router1 = RouterBuilder::new(
            ExampleRouterHandler {
                debug_out: debug_out1.clone(),
            },
            Some("127.0.0.1:8080".to_string()),
        );
        let router2: RouterBuilder<ExampleRouterHandler> = RouterBuilder::new(
            ExampleRouterHandler {
                debug_out: debug_out2.clone(),
            },
            Some("127.0.0.1:8081".to_string()),
        );

        tokio::spawn(async move {
            router2.listen().await?;
            Ok(())
        });
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

        let router1_client = router1.get_router_client();

        for _ in 0..3 {
            router1_client
                .queue_request::<ReadRequest>(
                    ReadRequest {
                        key: "test".as_bytes().to_vec(),
                    },
                    "127.0.0.1:8081".to_string(),
                )
                .await?;
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        }
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

        let debug_out1 = debug_out1.read().unwrap();
        assert_eq!(
            *debug_out1,
            vec![vec![1, 2, 3, 4], vec![1, 2, 3, 4], vec![1, 2, 3, 4]]
        );
        Ok(())
    }
}
