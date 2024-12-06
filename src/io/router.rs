use crate::io::write::write_message;
use crate::messages::message::AsAny;
use crate::messages::{
    message::{MessagePayload, MessageType},
    requests::{
        announce_shard_request::AnnounceShardRequest,
        get_client_shard_info_request::GetClientShardInfoRequest,
        query_version_request::QueryVersionRequest, read_request::ReadRequest,
        write_request::WriteRequest,
    },
    responses::{
        announce_shard_response::AnnounceShardResponse,
        get_client_shard_info_response::GetClientShardInfoResponse,
        query_version_response::QueryVersionResponse, read_response::ReadResponse,
        write_response::WriteResponse,
    },
};
use anyhow::{anyhow, Ok, Result};
use std::{cell::RefCell, collections::HashMap, sync::Arc};
use tokio::{
    io::{AsyncReadExt, Interest},
    net::{
        tcp::{OwnedReadHalf, OwnedWriteHalf, ReadHalf, WriteHalf},
        unix::SocketAddr,
        TcpListener, TcpStream,
    },
    task::JoinHandle,
};

use super::read::read_message;

/// Trait for handling callbacks to requests/responses from peers
pub trait RouterHandler: Send + Sync + 'static {
    /// Callback for handling new requests
    fn handle_announce_shard_request(&self, req: &AnnounceShardRequest) -> AnnounceShardResponse;

    fn handle_get_client_shard_info_request(
        req: GetClientShardInfoRequest,
    ) -> GetClientShardInfoResponse;

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

struct RouterBuilder<T>
where
    T: RouterHandler,
{
    pub handler: Arc<T>,
    /// Map of peer addresses to write sockets
    pub write_sockets: RefCell<HashMap<String, OwnedWriteHalf>>,
}

impl<T: RouterHandler> RouterBuilder<T> {
    pub fn new(handler: T) -> Self {
        Self {
            handler: Arc::new(handler),
            write_sockets: RefCell::new(HashMap::new()),
        }
    }

    /// Function for queueing outbound requests
    async fn queue_request<M: MessagePayload>(&self, req: M, peer: String) -> Result<()> {
        self.create_write_socket_if_needed(peer.clone()).await?;
        let mut write_sockets = self.write_sockets.borrow_mut();
        let mut write_socket = write_sockets.get_mut(&peer).unwrap();
        write_message(&mut write_socket, Box::new(req)).await?;
        Ok(())
    }

    /// Creates a write socket for a peer if it doesn't exist
    async fn create_write_socket_if_needed(self: &Self, peer: String) -> Result<()> {
        // check if peer is already connected
        if !self.write_sockets.borrow().contains_key(&peer) {
            let stream = TcpStream::connect(peer.clone()).await?;
            let (read, write) = stream.into_split();

            // push the write half to the map
            self.write_sockets.borrow_mut().insert(peer.clone(), write);

            // bind the read half to a background task
            let handler = self.handler.clone();
            tokio::spawn(async move {
                Self::listen_read_half_socket(handler, read).await?;
                Ok(())
            });
        }
        Ok(())
    }

    /// Listens for inbound requests on the read half of a socket from a peer
    async fn listen_read_half_socket<R: RouterHandler>(
        handler: Arc<R>,
        mut read: OwnedReadHalf,
    ) -> Result<()> {
        loop {
            read.readable().await?;
            let message = read_message(&mut read).await?;
            let peer = read.peer_addr()?.to_string();

            match message.get_message_type() {
                MessageType::AnnounceShard => {
                    let req = message
                        .as_any()
                        .downcast_ref::<AnnounceShardRequest>()
                        .unwrap();
                    let res = handler.handle_announce_shard_request(req);
                }
                _ => {
                    return Err(anyhow::anyhow!("unhandled message type"));
                }
            }
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
            self.write_sockets
                .borrow_mut()
                .insert(addr.to_string(), write);

            // bind the read half to a background task
            let handler = self.handler.clone();
            tokio::spawn(async move {
                Self::listen_read_half_socket(handler, read).await?;
                Ok(())
            });
        }
    }
}
