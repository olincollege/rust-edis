use anyhow::{Context, Result};
use messages::requests::{
    get_version_request::GetVersionRequest, query_version_request::QueryVersionRequest,
    write_request::WriteRequest,
};
use messages::responses::{
    get_version_response::{GetVersionResponse, GetVersionResponseError},
    query_version_response::QueryVersionResponse,
};
use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
mod messages;
use messages::message::{bytes_as_request_message, Message, MessagePayload, MessageType};

#[derive(Debug, Clone)]
struct WriteShard {
    data: Arc<Mutex<HashMap<String, String>>>, // Current key-value pairs
    version_history: Arc<Mutex<Vec<(String, String)>>>, // Global version history as key-value tuples
    current_version: Arc<Mutex<u64>>,                   // Tracks the latest version number
}

impl WriteShard {
    fn new() -> Self {
        WriteShard {
            data: Arc::new(Mutex::new(HashMap::new())),
            version_history: Arc::new(Mutex::new(Vec::new())),
            current_version: Arc::new(Mutex::new(0)),
        }
    }

    fn handle_write_request(&self, write_request: WriteRequest) -> Result<Vec<u8>> {
        let mut data = self.data.lock().unwrap();
        let mut version_history = self.version_history.lock().unwrap();
        let mut current_version = self.current_version.lock().unwrap();

        // Extract key and value from the request
        let key = String::from_utf8(write_request.key).context("Invalid UTF-8 in key")?;
        let value = String::from_utf8(write_request.value).context("Invalid UTF-8 in value")?;

        // Increment the version
        *current_version += 1;

        // Update the latest data
        data.insert(key.clone(), value.clone());

        // Append the change to the global version history
        version_history.push((key.clone(), value.clone()));

        Ok(b"Write successful".to_vec())
    }

    fn handle_query_version_request(&self, _request: QueryVersionRequest) -> Result<Vec<u8>> {
        let current_version = self.current_version.lock().unwrap();

        if *current_version > 0 {
            // Create the response with the latest version
            let response = QueryVersionResponse {
                version: *current_version,
            };
            response.serialize()
        } else {
            // No data available
            Ok(b"No data available".to_vec())
        }
    }

    fn handle_get_version_request(&self, request: GetVersionRequest) -> Result<Vec<u8>> {
        let version_history = self.version_history.lock().unwrap();

        // Find the requested version in the history
        if let Some((key, value)) = version_history.get(request.version as usize - 1) {
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

    fn handle_connection(&self, mut stream: TcpStream) -> Result<()> {
        let mut buffer = [0; 1024];
        let bytes_read = stream
            .read(&mut buffer)
            .context("Failed to read from stream")?;

        if bytes_read > 0 {
            let message_payload = bytes_as_request_message(&buffer[..bytes_read])
                .context("Failed to deserialize message")?;

            match message_payload.get_message_type() {
                MessageType::Write => {
                    let deserialized = Message::<WriteRequest>::deserialize(&buffer[..bytes_read])?;
                    let response = self.handle_write_request(deserialized.message_payload)?;
                    stream.write_all(&response[..])?;
                }
                MessageType::QueryVersion => {
                    let deserialized =
                        Message::<QueryVersionRequest>::deserialize(&buffer[..bytes_read])?;
                    let response =
                        self.handle_query_version_request(deserialized.message_payload)?;
                    stream.write_all(&response[..])?;
                }
                MessageType::GetVersion => {
                    let deserialized =
                        Message::<GetVersionRequest>::deserialize(&buffer[..bytes_read])?;
                    let response = self.handle_get_version_request(deserialized.message_payload)?;
                    stream.write_all(&response[..])?;
                }
                _ => {
                    stream.write_all(b"Unsupported message type")?;
                }
            }
        }

        Ok(())
    }

    fn run(&self, address: &str) -> Result<()> {
        let listener = TcpListener::bind(address).context("Failed to bind to address")?;
        println!("Write shard running on {}", address);

        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    let shard = self.clone();
                    std::thread::spawn(move || {
                        if let Err(e) = shard.handle_connection(stream) {
                            eprintln!("Error handling connection: {}", e);
                        }
                    });
                }
                Err(e) => eprintln!("Connection failed: {}", e),
            }
        }

        Ok(())
    }
}

fn main() -> Result<()> {
    let shard = WriteShard::new();
    shard.run("127.0.0.1:8080")
}
