use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    time::{sleep, Duration},
};

mod messages;
use messages::{
    get_version_request::GetVersionRequest,
    get_version_response::GetVersionResponse,
    message::{MessagePayload, MessageType},
    query_version_request::QueryVersionRequest,
    query_version_response::QueryVersionResponse,
};

pub struct ReadShard {
    reader_ip_port: String,
    writer_ip_port: String,
    current_version: u64,
    data: HashMap<u64, HashMap<String, String>>,
}

impl ReadShard {
    pub fn new(
        reader_ip_port: String,
        writer_ip_port: String,
        current_version: u64,
        data: HashMap<u64, HashMap<String, String>>,
    ) -> ReadShard {
        ReadShard {
            reader_ip_port,
            writer_ip_port,
            current_version,
            data,
        }
    }

    // Update only from the write shard. Will have to find a way to catchup from read shards too
    pub async fn update_shard_from_write(&mut self) -> Result<()> {
        let mut stream = TcpStream::connect(&self.writer_ip_port).await?;

        loop {
            sleep(Duration::from_secs(3)).await;

            let request = QueryVersionRequest {};
            let serialized_request = request.serialize()?;
            stream.write_all(&serialized_request).await?;

            let mut buffer = [0u8; 1024];
            let n = stream.read(&mut buffer).await?;
            if n == 0 {
                continue;
            }

            let response = QueryVersionResponse::deserialize(&buffer[0..n])?;

            if response.version > self.current_version {
                let request = GetVersionRequest {
                    version: self.current_version + 1,
                };
                let serialized_request = request.serialize()?;
                stream.write_all(&serialized_request).await?;

                let mut buffer = [0u8; 1024];
                let n = stream.read(&mut buffer).await?;
                if n == 0 {
                    continue;
                }

                let response = GetVersionResponse::deserialize(&buffer[0..n])?;

                if response.error != 0 {
                    let mut present_data = self
                        .data
                        .get(&self.current_version)
                        .cloned()
                        .unwrap_or_else(HashMap::new);

                    present_data.insert(
                        String::from_utf8(response.key)?,
                        String::from_utf8(response.value)?,
                    );
                    self.current_version += 1;

                    self.data.insert(self.current_version, present_data);
                }
            }
        }
    }

    pub async fn get_value(&self, key: &str) -> Option<&String> {
        let present_data = self.data.get(&self.current_version).unwrap();
        present_data.get(key)
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind("127.0.0.1:0").await?;

    let reader_addr = listener.local_addr().unwrap();
    let writer_addr = "127.0.0.1:0".to_string();

    let shard = Arc::new(Mutex::new(ReadShard::new(
        reader_addr.to_string(),
        writer_addr,
        0,
        HashMap::new(),
    )));

    let shard_clone = Arc::clone(&shard);
    tokio::spawn(async move {
        let mut shard_lock = shard_clone.lock().await;
        shard_lock.update_shard_from_write().await.unwrap();
    });

    Ok(())
}
