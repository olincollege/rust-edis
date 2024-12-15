use crate::io::write::write_message;
use crate::messages::message::{AsAny, Message};
use crate::messages::requests::get_shared_peers_request::GetSharedPeersRequest;
use crate::messages::responses::get_shared_peers_response::GetSharedPeersResponse;
use crate::messages::{
    message::{MessagePayload, MessageType},
    requests::{
        announce_shard_request::AnnounceShardRequest,
        get_client_shard_info_request::GetClientShardInfoRequest,
        get_version_request::GetVersionRequest, query_version_request::QueryVersionRequest,
        read_request::ReadRequest, write_request::WriteRequest,
    },
    responses::{
        announce_shard_response::AnnounceShardResponse,
        get_client_shard_info_response::GetClientShardInfoResponse,
        get_version_response::GetVersionResponse, query_version_response::QueryVersionResponse,
        read_response::ReadResponse, write_response::WriteResponse,
    },
};
use anyhow::{Ok, Result};
use async_recursion::async_recursion;
use scc::HashMap;
use std::net::SocketAddr::{V4, V6};
use std::net::{Ipv6Addr, SocketAddrV6};
use std::sync::Arc;
use tokio::net::{tcp::OwnedReadHalf, TcpListener, TcpStream};

use super::read::read_message;

/// Trait for handling callbacks to requests/responses from peers
pub trait RouterHandler: Send + Sync + 'static {
    /// Callback for handling new requests
    fn handle_announce_shard_request(&self, req: &AnnounceShardRequest) -> AnnounceShardResponse;

    fn handle_get_client_shard_info_request(
        &self,
        req: &GetClientShardInfoRequest,
    ) -> GetClientShardInfoResponse;

    fn handle_query_version_request(&self, req: &QueryVersionRequest) -> QueryVersionResponse;

    fn handle_read_request(&self, req: &ReadRequest) -> ReadResponse;

    fn handle_write_request(&self, req: &WriteRequest) -> WriteResponse;

    fn handle_get_shared_peers_request(
        &self,
        req: &GetSharedPeersRequest,
    ) -> GetSharedPeersResponse;

    fn handle_get_version_request(&self, req: &GetVersionRequest) -> GetVersionResponse;

    /// Callbacks for handling responses to outbound requests
    fn handle_announce_shard_response(&self, res: &AnnounceShardResponse);

    fn handle_get_client_shard_info_response(&self, res: &GetClientShardInfoResponse);

    fn handle_query_version_response(&self, res: &QueryVersionResponse);

    fn handle_get_version_response(&self, res: &GetVersionResponse);

    fn handle_read_response(&self, res: &ReadResponse);

    fn handle_write_response(&self, res: &WriteResponse);

    fn handle_get_shared_peers_response(&self, res: &GetSharedPeersResponse);
}

pub struct RouterBuilder<H: RouterHandler> {
    pub handler: Arc<H>,
    /// Map of peer addresses to write sockets
    pub write_sockets: Arc<HashMap<SocketAddrV6, tokio::net::tcp::OwnedWriteHalf>>,

    pub bind_addr: Option<SocketAddrV6>,

    listener: Option<TcpListener>,
}

/// Owned struct returned from RouterBuilder that allows for
/// the implementer of RouterHandler to send outbound requests
/// Returned from get_client_router() in RouterHandler
pub struct RouterClient<H: RouterHandler> {
    pub handler: Arc<H>,

    /// Map of peer addresses to write sockets
    /// ownership is retained on a per-key basis under async lock
    pub write_sockets: Arc<HashMap<SocketAddrV6, tokio::net::tcp::OwnedWriteHalf>>,
}

impl<H: RouterHandler> RouterClient<H> {
    /// Function for queueing outbound requests
    pub async fn queue_request<M: MessagePayload>(&self, req: M, peer: SocketAddrV6) -> Result<()> {
        RouterBuilder::create_write_socket_if_needed(
            self.write_sockets.clone(),
            self.handler.clone(),
            peer.clone(),
        )
        .await?;
        let mut write_socket = self.write_sockets.get_async(&peer).await.unwrap();
        write_message(
            &mut write_socket,
            Message {
                is_request: true,
                message_type: req.get_message_type(),
                message_payload: req,
            },
        )
        .await?;
        Ok(())
    }
}

unsafe impl<H: RouterHandler> Send for RouterBuilder<H> {}

impl<H: RouterHandler> RouterBuilder<H> {
    pub fn new(handler: H, bind_addr: Option<SocketAddrV6>) -> Self {
        Self {
            handler: Arc::new(handler),
            write_sockets: Arc::new(scc::HashMap::new()),
            bind_addr,
            listener: None,
        }
    }

    pub fn get_router_client(&self) -> RouterClient<H> {
        RouterClient {
            handler: self.handler.clone(),
            write_sockets: self.write_sockets.clone(),
        }
    }

    pub fn get_handler_arc(&self) -> Arc<H> {
        self.handler.clone()
    }

    /// Function for queueing outbound responses
    async fn queue_response<M: MessagePayload>(
        write_sockets: Arc<HashMap<SocketAddrV6, tokio::net::tcp::OwnedWriteHalf>>,
        handler: Arc<H>,
        res: M,
        peer: SocketAddrV6,
    ) -> Result<()> {
        Self::create_write_socket_if_needed(write_sockets.clone(), handler.clone(), peer.clone())
            .await?;
        let mut write_socket = write_sockets.get_async(&peer).await.unwrap();
        write_message(
            &mut write_socket,
            Message {
                is_request: false,
                message_type: res.get_message_type(),
                message_payload: res,
            },
        )
        .await?;
        Ok(())
    }

    /// Creates a write socket for a peer if it doesn't exist
    /// I don't understand the async recursion problem but something to do with how async builds state machines
    /// https://www.reddit.com/r/rust/comments/kbu6bs/async_recursive_function_in_rust_using_futures/
    #[async_recursion]
    async fn create_write_socket_if_needed(
        write_sockets: Arc<HashMap<SocketAddrV6, tokio::net::tcp::OwnedWriteHalf>>,
        handler: Arc<H>,
        peer: SocketAddrV6,
    ) -> Result<()> {
        // check if peer is already connected
        if !write_sockets.contains_async(&peer).await {
            //println!("creating!");
            let stream = TcpStream::connect(peer.clone()).await?;
            let (read, write) = stream.into_split();

            // push the write half to the map
            write_sockets
                .insert_async(peer.clone(), write)
                .await
                .map_err(|e| anyhow::anyhow!("Failed to insert write socket: {:?}", e))?;

            // bind the read half to a background task
            let handler = handler.clone();
            let write_sockets = write_sockets.clone();
            tokio::spawn(async move {
                Self::listen_read_half_socket(write_sockets, handler, read).await?;
                Ok(())
            });
        }
        Ok(())
    }

    /// Listens for inbound requests on the read half of a socket from a peer
    async fn listen_read_half_socket(
        write_sockets: Arc<HashMap<SocketAddrV6, tokio::net::tcp::OwnedWriteHalf>>,
        handler: Arc<H>,
        mut read: OwnedReadHalf,
    ) -> Result<()> {
        loop {
            read.readable().await?;
            //println!("done waiting for read");
            let message = read_message(&mut read).await?;
            let peer = read.peer_addr()?;

            match peer {
                V6(peer) => {
                    let msg_type = message.get_message_type();
                    // println!("(router) handling new message of type {:?}", msg_type);
                    match message.is_request() {
                        true => {
                            match message.get_message_type() {
                                MessageType::AnnounceShard => {
                                    let req = message
                                        .as_ref()
                                        .as_any()
                                        .downcast_ref::<AnnounceShardRequest>()
                                        .unwrap();
                                    Self::queue_response::<AnnounceShardResponse>(
                                        write_sockets.clone(),
                                        handler.clone(),
                                        handler.handle_announce_shard_request(req),
                                        peer,
                                    )
                                    .await?;
                                }
                                MessageType::GetClientShardInfo => {
                                    let req = message
                                        .as_ref()
                                        .as_any()
                                        .downcast_ref::<GetClientShardInfoRequest>()
                                        .unwrap();
                                    Self::queue_response::<GetClientShardInfoResponse>(
                                        write_sockets.clone(),
                                        handler.clone(),
                                        handler.handle_get_client_shard_info_request(req),
                                        peer,
                                    )
                                    .await?;
                                }
                                MessageType::QueryVersion => {
                                    let req = message
                                        .as_ref()
                                        .as_any()
                                        .downcast_ref::<QueryVersionRequest>()
                                        .unwrap();
                                    Self::queue_response::<QueryVersionResponse>(
                                        write_sockets.clone(),
                                        handler.clone(),
                                        handler.handle_query_version_request(req),
                                        peer,
                                    )
                                    .await?;
                                }
                                MessageType::Read => {
                                    let req = message
                                        .as_ref()
                                        .as_any()
                                        .downcast_ref::<ReadRequest>()
                                        .unwrap();
                                    Self::queue_response::<ReadResponse>(
                                        write_sockets.clone(),
                                        handler.clone(),
                                        handler.handle_read_request(req),
                                        peer,
                                    )
                                    .await?;
                                }
                                MessageType::Write => {
                                    let req = message
                                        .as_ref()
                                        .as_any()
                                        .downcast_ref::<WriteRequest>()
                                        .unwrap();
                                    Self::queue_response::<WriteResponse>(
                                        write_sockets.clone(),
                                        handler.clone(),
                                        handler.handle_write_request(req),
                                        peer,
                                    )
                                    .await?;
                                }
                                MessageType::GetSharedPeers => {
                                    let req = message
                                        .as_ref()
                                        .as_any()
                                        .downcast_ref::<GetSharedPeersRequest>()
                                        .unwrap();
                                    Self::queue_response::<GetSharedPeersResponse>(
                                        write_sockets.clone(),
                                        handler.clone(),
                                        handler.handle_get_shared_peers_request(req),
                                        peer,
                                    )
                                    .await?;
                                }
                                MessageType::GetVersion => {
                                    let req = message
                                        .as_ref()
                                        .as_any()
                                        .downcast_ref::<GetVersionRequest>()
                                        .unwrap();
                                    Self::queue_response::<GetVersionResponse>(
                                        write_sockets.clone(),
                                        handler.clone(),
                                        handler.handle_get_version_request(req),
                                        peer,
                                    )
                                    .await?;
                                }
                            };
                        }
                        false => match message.get_message_type() {
                            MessageType::AnnounceShard => {
                                println!("(router): handling AnnounceShard message");
                                let res = message
                                    .as_ref()
                                    .as_any()
                                    .downcast_ref::<AnnounceShardResponse>()
                                    .unwrap();
                                handler.handle_announce_shard_response(res)
                            }
                            MessageType::GetClientShardInfo => {
                                let res = message
                                    .as_ref()
                                    .as_any()
                                    .downcast_ref::<GetClientShardInfoResponse>()
                                    .unwrap();
                                handler.handle_get_client_shard_info_response(res)
                            }
                            MessageType::QueryVersion => {
                                let res = message
                                    .as_ref()
                                    .as_any()
                                    .downcast_ref::<QueryVersionResponse>()
                                    .unwrap();
                                handler.handle_query_version_response(res)
                            }
                            MessageType::Read => {
                                let res = message
                                    .as_ref()
                                    .as_any()
                                    .downcast_ref::<ReadResponse>()
                                    .unwrap();
                                handler.handle_read_response(res)
                            }
                            MessageType::Write => {
                                let res = message
                                    .as_ref()
                                    .as_any()
                                    .downcast_ref::<WriteResponse>()
                                    .unwrap();
                                handler.handle_write_response(res)
                            }
                            MessageType::GetVersion => {
                                let res = message
                                    .as_ref()
                                    .as_any()
                                    .downcast_ref::<GetVersionResponse>()
                                    .unwrap();
                                handler.handle_get_version_response(res)
                            }
                            MessageType::GetSharedPeers => {
                                let res = message
                                    .as_ref()
                                    .as_any()
                                    .downcast_ref::<GetSharedPeersResponse>()
                                    .unwrap();
                                handler.handle_get_shared_peers_response(res)
                            }
                        },
                    };
                }
                V4(_peer) => {
                    println!("ignoring ipv4 peer, please use ipv6")
                }
            }
        }
    }

    pub async fn bind(&mut self) -> Result<SocketAddrV6> {
        let listener = TcpListener::bind(self.bind_addr.unwrap_or(SocketAddrV6::new(
            Ipv6Addr::LOCALHOST,
            0,
            0,
            0,
        )))
        .await?;

        let addr = listener.local_addr()?;
        println!("listening on {}", addr);

        match addr {
            V6(addr) => {
                self.listener = Some(listener);
                Ok(addr)
            }
            V4(_) => {
                anyhow::bail!("IPv4 addresses are not supported, please use IPv6")
            }
        }
    }
    /// Makes the router start listening for inbound requests
    pub async fn listen(&mut self) -> Result<()> {
        loop {
            //let (mut socket, addr) = Arc::new(listener.accept().await?);
            let listener = self.listener.as_mut();

            match listener {
                Some(listener) => {
                    let (socket, addr) = listener.accept().await?;
                    match addr {
                        V6(addr) => {
                            let (read, write) = socket.into_split();

                            // new peer discovered, add to our list of write sockets
                            self.write_sockets
                                .insert_async(addr.clone(), write)
                                .await
                                .map_err(|_e| anyhow::anyhow!("Failed to insert write socket"))?;

                            // bind the read half to a background task
                            let handler = self.handler.clone();
                            let write_sockets = self.write_sockets.clone();
                            tokio::spawn(async move {
                                Self::listen_read_half_socket(write_sockets, handler, read).await?;
                                Ok(())
                            });
                        }
                        V4(_addr) => {
                            println!("ignoring ipv4 connection, please use ipv6");
                        }
                    }
                }
                None => {
                    println!("bind() needs to be called before listen!");
                    anyhow::bail!("bind() needs to be called before listen!");
                }
            }
        }
    }
}
