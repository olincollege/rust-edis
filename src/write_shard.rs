use anyhow::{Context, Result};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;
mod messages;
use messages::message::MessagePayload;
use messages::message::{Message, MessageType};
use messages::requests::{
    get_version_request::GetVersionRequest, query_version_request::QueryVersionRequest,
    write_request::WriteRequest,
};
use messages::responses::{
    get_version_response::{GetVersionResponse, GetVersionResponseError},
    query_version_response::QueryVersionResponse,
};

#[derive(Debug)]
struct WriteShard {
    data: HashMap<String, String>,
    version_history: Vec<(String, String)>,
    current_version: u64,
}

impl WriteShard {
    fn new() -> Self {
        WriteShard {
            data: HashMap::new(),
            version_history: Vec::new(),
            current_version: 0,
        }
    }

    async fn handle_write_request(&mut self, write_request: WriteRequest) -> Result<Vec<u8>> {
        // Extract key and value from the request
        let key = String::from_utf8(write_request.key).context("Invalid UTF-8 in key")?;
        let value = String::from_utf8(write_request.value).context("Invalid UTF-8 in value")?;

        // Increment the version
        self.current_version += 1;

        // Update the latest data
        self.data.insert(key.clone(), value.clone());

        // Append the change to the global version history
        self.version_history.push((key.clone(), value.clone()));

        Ok(b"Write successful".to_vec())
    }

    async fn handle_query_version_request(&self, _request: QueryVersionRequest) -> Result<Vec<u8>> {
        if self.current_version > 0 {
            // Create the response with the latest version
            let response = QueryVersionResponse {
                version: self.current_version,
            };
            response.serialize()
        } else {
            // No data available
            Ok(b"No data available".to_vec())
        }
    }

    async fn handle_get_version_request(&self, request: GetVersionRequest) -> Result<Vec<u8>> {
        // Find the requested version in the history
        if let Some((key, value)) = self.version_history.get(request.version as usize - 1) {
            // Create a successful response
            let response = GetVersionResponse {
                error: GetVersionResponseError::NoError as u8,
                key: key.clone().into_bytes(),
                value: value.clone().into_bytes(),
            };
            return response.serialize();
        }

        // If the version is not found, return an error response
        let response = GetVersionResponse {
            error: GetVersionResponseError::KeyNotFound as u8,
            key: Vec::new(),   // No key in the error case
            value: Vec::new(), // No value in the error case
        };
        response.serialize()
    }

    async fn handle_connection(&mut self, mut stream: TcpStream) -> Result<()> {
        let mut buffer = [0; 1024];
        let bytes_read = stream
            .read(&mut buffer)
            .await
            .context("Failed to read from stream")?;

        if bytes_read > 0 {
            // Get message type directly from the buffer
            let message_type = MessageType::try_from(buffer[4])
                .map_err(|_| anyhow::anyhow!("invalid message type"))?;

            match message_type {
                MessageType::Write => {
                    let deserialized = Message::<WriteRequest>::deserialize(&buffer[..bytes_read])?;
                    let response = self
                        .handle_write_request(deserialized.message_payload)
                        .await?;
                    stream.write_all(&response[..]).await?;
                }
                MessageType::QueryVersion => {
                    let deserialized =
                        Message::<QueryVersionRequest>::deserialize(&buffer[..bytes_read])?;
                    let response = self
                        .handle_query_version_request(deserialized.message_payload)
                        .await?;
                    stream.write_all(&response[..]).await?;
                }
                MessageType::GetVersion => {
                    let deserialized =
                        Message::<GetVersionRequest>::deserialize(&buffer[..bytes_read])?;
                    let response = self
                        .handle_get_version_request(deserialized.message_payload)
                        .await?;
                    stream.write_all(&response[..]).await?;
                }
                _ => {
                    stream.write_all(b"Unsupported message type").await?;
                }
            }
        }

        Ok(())
    }
}

// Derive Clone for thread-safe shared state
impl Clone for WriteShard {
    fn clone(&self) -> Self {
        WriteShard {
            data: self.data.clone(),
            version_history: self.version_history.clone(),
            current_version: self.current_version,
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let shard = Arc::new(Mutex::new(WriteShard::new()));
    let address = "127.0.0.1:8080";

    let listener = TcpListener::bind(address).await?;
    println!("Write shard running on {}", address);

    tokio::select! {
        _ = async {
            loop {
                let (stream, _) = listener.accept().await.expect("Failed to accept connection");
                let shard_clone = Arc::clone(&shard);

                tokio::spawn(async move {
                    let mut shard_lock = shard_clone.lock().await;
                    if let Err(e) = shard_lock.handle_connection(stream).await {
                        eprintln!("Error handling connection: {}", e);
                    }
                });
            }
        } => {}
        _ = tokio::signal::ctrl_c() => {
            println!("Shutting down gracefully...");
        }
    }

    Ok(())
}
