use async_smux::{Mux, MuxConfig, MuxStream};
use anyhow::Result;
use tokio::{io::AsyncReadExt, net::TcpStream};
use std::io::Write;


use crate::messages::message::{Message, MessagePayload};

pub async fn read_message(mut stream: MuxStream<TcpStream>) -> Result<Box<dyn MessagePayload>> {
    let mut buffer = [0; 4096];
    let mut buffer_idx = 0;

    // read the total length and then proceed with the rest once the message size is known
    let total_length = match stream.read_u32_le().await {
        Ok(n) => n,
        Err(e) => {
            return Err(anyhow::anyhow!("failed to read total length: {}", e));
        }
    };

    buffer[0..4].copy_from_slice(&total_length.to_le_bytes());
    buffer_idx += 4;
    let total_length = total_length as usize;

    // read the rest of the message
    while let Ok(bytes_read) = stream.read(&mut buffer[buffer_idx..]).await {
        match bytes_read {
            0 => {
                return Err(anyhow::anyhow!("connection closed"));
            }
            _ => {
                match std::cmp::Ordering::from((buffer_idx + bytes_read).cmp(&total_length)) {
                    std::cmp::Ordering::Greater => {
                        return Err(anyhow::anyhow!("invalid message length"));
                    }
                    std::cmp::Ordering::Equal => {
                        // all good
                        break;
                    }
                    std::cmp::Ordering::Less => {
                        // keep reading
                        buffer_idx += bytes_read;
                    }
                }
            }
        }
    }

    // deserialize the message
    let message = Message::<Box<dyn MessagePayload>>::deserialize(&buffer[0..total_length])?;

    return Ok(message.message_payload);
}
