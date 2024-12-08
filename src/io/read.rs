use anyhow::Result;
use async_smux::MuxStream;
use tokio::{
    io::AsyncReadExt,
    net::{tcp::OwnedReadHalf, TcpStream},
};

use crate::messages::message::{bytes_as_message, MessagePayload};

pub async fn read_message(stream: &mut OwnedReadHalf) -> Result<Box<dyn MessagePayload>> {
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
    
    println!("reading message length: {total_length}");

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
    bytes_as_message(&buffer[0..total_length])
}
