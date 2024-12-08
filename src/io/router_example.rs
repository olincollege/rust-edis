use crate::messages::{requests::{announce_shard_request::AnnounceShardRequest, get_client_shard_info_request::GetClientShardInfoRequest, get_shared_peers_request::GetSharedPeersRequest, query_version_request::QueryVersionRequest, read_request::ReadRequest, write_request::WriteRequest}, responses::{announce_shard_response::AnnounceShardResponse, get_client_shard_info_response::GetClientShardInfoResponse, get_shared_peers_response::GetSharedPeersResponse, query_version_response::QueryVersionResponse, read_response::ReadResponse, write_response::WriteResponse}};
use crate::io::router::{RouterBuilder, RouterHandler};
struct ExampleRouterHandler {

}

impl RouterHandler for ExampleRouterHandler {
    /// Callback for handling new requests
    fn handle_announce_shard_request(&self, req: &AnnounceShardRequest) -> AnnounceShardResponse {
        AnnounceShardResponse {
            writer_number: 0,   
        }
    }

    fn handle_get_client_shard_info_request(&self, req: &GetClientShardInfoRequest) -> GetClientShardInfoResponse {
        GetClientShardInfoResponse {
            num_write_shards: 0,
            write_shard_info: vec![],
            read_shard_info: vec![],
        }
    }

    fn handle_query_version_request(&self, req: &QueryVersionRequest) -> QueryVersionResponse {
        QueryVersionResponse {
            version: 0,
        }
    }

    fn handle_read_request(&self, req: &ReadRequest) -> ReadResponse {
        ReadResponse {
            value: vec![],
        }
    }

    fn handle_write_request(&self, req: &WriteRequest) -> WriteResponse {
        WriteResponse {
            error: 0,
        }
    }

    fn handle_get_shared_peers_request(&self, req: &GetSharedPeersRequest) -> GetSharedPeersResponse {
        GetSharedPeersResponse {
            peer_ips: vec![],
        }
    }

    /// Callbacks for handling responses to outbound requests
    fn handle_announce_shard_response(&self, res: &AnnounceShardResponse) {

    }

    fn handle_get_client_shard_info_response(&self, res: &GetClientShardInfoResponse) {
        
    }

    fn handle_query_version_response(&self, res: &QueryVersionResponse) {

    }

    fn handle_read_response(&self, res: &ReadResponse) {

    }

    fn handle_write_response(&self, res: &WriteResponse) {

    }

    fn handle_get_shared_peers_response(&self, res: &GetSharedPeersResponse) {

    }   

}


mod test {
    use anyhow::{Ok,Result};
    use crate::{io::{router::RouterBuilder, router_example::ExampleRouterHandler}, messages::requests::{query_version_request::QueryVersionRequest, read_request::ReadRequest}};


    #[tokio::test]
    async fn test_example_router() -> Result<()> {
        let router1 = RouterBuilder::new(ExampleRouterHandler {}, Some("127.0.0.1:8080".to_string()));
        let router2: RouterBuilder<ExampleRouterHandler> = RouterBuilder::new(ExampleRouterHandler {}, Some("127.0.0.1:8081".to_string()));

        tokio::spawn(async move {
            router2.listen().await?;
            Ok(())
        });
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

        for _ in 0..3 {
            router1.queue_request::<ReadRequest>(ReadRequest {
                key: "test".as_bytes().to_vec()
            }, "127.0.0.1:8081".to_string()).await?;
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        }
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

        Ok(())
    }
}