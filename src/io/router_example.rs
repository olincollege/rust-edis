#[cfg(test)]
mod test {

    use crate::integration::test_setup;
    use crate::io::router::RouterHandler;
    use crate::messages::requests::announce_shard_request::AnnounceShardRequest;
    use crate::messages::requests::get_client_shard_info_request::GetClientShardInfoRequest;
    use crate::messages::requests::get_shared_peers_request::GetSharedPeersRequest;
    use crate::messages::requests::query_version_request::QueryVersionRequest;
    use crate::messages::requests::write_request::WriteRequest;
    use crate::messages::responses::announce_shard_response::AnnounceShardResponse;
    use crate::messages::responses::get_client_shard_info_response::GetClientShardInfoResponse;
    use crate::messages::responses::get_shared_peers_response::GetSharedPeersResponse;
    use crate::messages::responses::query_version_response::QueryVersionResponse;
    use crate::messages::responses::read_response::ReadResponse;
    use crate::messages::responses::write_response::WriteResponse;
    use crate::{io::router::RouterBuilder, messages::requests::read_request::ReadRequest};
    use anyhow::Result;
    use serial_test::serial;
    use std::net::{Ipv6Addr, SocketAddrV6};
    use std::sync::{Arc, RwLock};

    struct ExampleRouterHandler {
        debug_out: Arc<RwLock<Vec<Vec<u8>>>>,
    }

    impl RouterHandler for ExampleRouterHandler {
        /// Callback for handling new requests
        fn handle_announce_shard_request(
            &self,
            _req: &AnnounceShardRequest,
        ) -> AnnounceShardResponse {
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
            ReadResponse {
                value: vec![1, 2, 3, 4],
                key: b"testkey".to_vec(),
                error: 0,
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

        /// Callbacks for handling responses to outbound requests
        fn handle_announce_shard_response(&self, _res: &AnnounceShardResponse) {
            unimplemented!()
        }

        fn handle_get_client_shard_info_response(&self, _res: &GetClientShardInfoResponse) {
            unimplemented!()
        }

        fn handle_query_version_response(&self, _res: &QueryVersionResponse) {
            unimplemented!()
        }

        fn handle_read_response(&self, res: &ReadResponse) {
            self.debug_out.write().unwrap().push(res.value.clone());
        }

        fn handle_write_response(&self, _res: &WriteResponse) {
            unimplemented!()
        }

        fn handle_get_shared_peers_response(&self, _res: &GetSharedPeersResponse) {
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
    }

    #[tokio::test]
    #[serial]
    async fn test_example_router() -> Result<()> {
        test_setup::setup_test().await;

        let debug_out1: Arc<RwLock<Vec<Vec<u8>>>> = Arc::new(RwLock::new(Vec::new()));
        let debug_out2: Arc<RwLock<Vec<Vec<u8>>>> = Arc::new(RwLock::new(Vec::new()));

        let router1 = RouterBuilder::new(
            ExampleRouterHandler {
                debug_out: debug_out1.clone(),
            },
            Some(SocketAddrV6::new(Ipv6Addr::LOCALHOST, 8080, 0, 0)),
        );
        let mut router2: RouterBuilder<ExampleRouterHandler> = RouterBuilder::new(
            ExampleRouterHandler {
                debug_out: debug_out2.clone(),
            },
            Some(SocketAddrV6::new(Ipv6Addr::LOCALHOST, 8081, 0, 0)),
        );

        tokio::spawn(async move {
            router2.bind().await?;
            router2.listen().await?;
            anyhow::Ok(())
        });
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

        let router1_client = router1.get_router_client();

        for _ in 0..3 {
            router1_client
                .queue_request::<ReadRequest>(
                    ReadRequest {
                        key: "test".as_bytes().to_vec(),
                    },
                    SocketAddrV6::new(Ipv6Addr::LOCALHOST, 8081, 0, 0),
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

        test_setup::test_teardown().await;
        Ok(())
    }
}
