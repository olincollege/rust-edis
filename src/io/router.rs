
use crate::messages::{requests::{
    announce_shard_request::AnnounceShardRequest,
    get_client_shard_info_request::GetClientShardInfoRequest,
    query_version_request::QueryVersionRequest,
    read_request::ReadRequest,
    write_request::WriteRequest
}, responses::{
    announce_shard_response::AnnounceShardResponse,
    get_client_shard_info_response::GetClientShardInfoResponse,
    query_version_response::QueryVersionResponse,
    read_response::ReadResponse,
    write_response::WriteResponse
}};
use anyhow::{Result, anyhow, Ok};
use tokio::{io::{AsyncReadExt, Interest, WriteHalf}, net::{unix::SocketAddr, TcpListener, TcpStream}};
use std::{cell::RefCell, collections::HashMap};


/// Trait for handling callbacks to inbound/outbound requests
pub trait RouterHandler {
    /// Callback for handling new requests
    fn handle_announce_shard_request(req: AnnounceShardRequest) -> AnnounceShardResponse;

    fn handle_get_client_shard_info_request(req: GetClientShardInfoRequest) -> GetClientShardInfoResponse;

    fn handle_query_version_request(req: QueryVersionRequest) -> QueryVersionResponse;

    fn handle_read_request(req: ReadRequest) -> ReadResponse;

    fn handle_write_request(req: WriteRequest) -> WriteResponse;

    /// Functions for queueing outbound requests
    fn queue_announce_shard_request(req: AnnounceShardRequest);

    fn queue_get_client_shard_info_request(req: GetClientShardInfoRequest);

    fn queue_query_version_request(req: QueryVersionRequest);

    fn queue_read_request(req: ReadRequest);

    fn queue_write_request(req: WriteRequest);

    /// Callbacks for handling responses to outbound requests
    fn handle_announce_shard_response(res: AnnounceShardResponse);

    fn handle_get_client_shard_info_response(res: GetClientShardInfoResponse);

    fn handle_query_version_response(res: QueryVersionResponse);

    fn handle_read_response(res: ReadResponse);

    fn handle_write_response(res: WriteResponse);
}


struct RouterBuilder<T: RouterHandler> {
    handler: T,
    write_sockets: RefCell<HashMap<String, WriteHalf<TcpStream>>>,
}

impl<T: RouterHandler> RouterBuilder<T> {
    pub fn new(handler: T) -> Self {
        Self { handler, write_sockets: RefCell::new(HashMap::new()) }
    }

    /// Makes the router start listening for inbound requests
    async fn listen(&self) -> Result<()> {
        let listener = TcpListener::bind("127.0.0.1:8080").await?;

        loop {
            let (mut socket, addr) = listener.accept().await?;
            let (read, write) = tokio::io::split(socket);
            
            // new peer discovered, add to our list of write sockets
            self.write_sockets.borrow_mut().insert(addr.to_string(), write);

            tokio::spawn(async move {
                let mut buf = [0; 1024];
                //read.read(&mut buf).await?;
                Ok(())
            });
        }
        Ok(())
    }
}