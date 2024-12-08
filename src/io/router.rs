
use crate::messages::{message::MessagePayload, requests::{
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
use crate::io::write::write_message;
use anyhow::{Result, anyhow, Ok};
use tokio::{io::{AsyncReadExt, Interest}, net::{tcp::{OwnedReadHalf, OwnedWriteHalf, ReadHalf, WriteHalf}, unix::SocketAddr, TcpListener, TcpStream}, task::JoinHandle};
use std::{cell::RefCell, collections::HashMap, sync::Arc};

/// Trait for handling callbacks to requests/responses from peers 
pub trait RouterHandler {
    /// Callback for handling new requests
    fn handle_announce_shard_request(req: AnnounceShardRequest) -> AnnounceShardResponse;

    fn handle_get_client_shard_info_request(req: GetClientShardInfoRequest) -> GetClientShardInfoResponse;

    fn handle_query_version_request(req: QueryVersionRequest) -> QueryVersionResponse;

    fn handle_read_request(req: ReadRequest) -> ReadResponse;

    fn handle_write_request(req: WriteRequest) -> WriteResponse;

  
    /// Callbacks for handling responses to outbound requests
    fn handle_announce_shard_response(res: AnnounceShardResponse);

    fn handle_get_client_shard_info_response(res: GetClientShardInfoResponse);

    fn handle_query_version_response(res: QueryVersionResponse);

    fn handle_read_response(res: ReadResponse);

    fn handle_write_response(res: WriteResponse);
}


struct RouterBuilder<T: RouterHandler> {
    handler: T,
    /// Map of peer addresses to write sockets
    write_sockets: RefCell<HashMap<String, OwnedWriteHalf>>,

}

impl<T: RouterHandler> RouterBuilder<T> {
    pub fn new(handler: T) -> Self {
        Self { handler, write_sockets: RefCell::new(HashMap::new()) }
    }

    /// Function for queueing outbound requests
    async fn queue_request<M: MessagePayload>(self: &Self, req: M, peer: String) -> Result<()> {
        self.create_write_socket_if_needed(peer).await?;
        let write_socket = self.write_sockets.borrow().get(&peer).unwrap();
        write_message(write_socket, Box::new(req)).await?;
        Ok(())
    }
    
    /// Creates a write socket for a peer if it doesn't exist
    async fn create_write_socket_if_needed(&self, peer: String) -> Result<()> {
        // check if peer is already connected
        if !self.write_sockets.borrow().contains_key(&peer) {
            let stream = TcpStream::connect(peer.clone()).await?;
            let (read, write) = stream.into_split();

            // push the write half to the map
            self.write_sockets.borrow_mut().insert(peer.clone(), write);

            // bind the read half to a background task
            tokio::spawn(async move {
                Self::listen_read_half_socket(read).await?;
                Ok(())
            });
        }
        Ok(())
    }

    /// Listens for inbound requests on the read half of a socket from a peer
    async fn listen_read_half_socket(mut read: OwnedReadHalf) -> Result<()> {
        let mut buf = [0; 1024];
        loop {
            read.readable().await?;
            read.read(&mut buf).await?;
            read.peer_addr()?;
        }
    }

    /// Makes the router start listening for inbound requests 
    async fn listen(&self) -> Result<()> {
        let listener = TcpListener::bind("127.0.0.1:8080").await?;

        loop {
            //let (mut socket, addr) = Arc::new(listener.accept().await?);
            let (socket, addr) = listener.accept().await?;

            let (read, write) = socket.into_split();
            
            // new peer discovered, add to our list of write sockets
            self.write_sockets.borrow_mut().insert(addr.to_string(), write);
            
            // bind the read half to a background task
            tokio::spawn(async move {
                Self::listen_read_half_socket(read).await?;
                Ok(())
            });
        }
        Ok(())
    }
}