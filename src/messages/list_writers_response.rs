use crate::messages::message::{MessagePayload, MessageType};
use anyhow::{Context, Result};

pub struct ListWritersResponse {
    pub writers: Vec<([u8; 16], u16)>, // Each writer is a tuple of 16-byte IP and 2-byte port
}

/// Layout of the ListWritersResponse
/// | 2 bytes (writer count) | 18 bytes per writer | N writers |
/// |       Writer Count      |     IP (16 bytes)   | Port (2 bytes) |
impl MessagePayload for ListWritersResponse {
    fn get_message_type(&self) -> MessageType {
        MessageType::ListWriters
    }

    fn serialize(&self) -> Result<Vec<u8>> {
        let mut buffer = Vec::new();

        // Add the number of writers (2 bytes, little-endian)
        let count = u16::try_from(self.writers.len()).context("too many writers")?;
        buffer.extend_from_slice(&count.to_le_bytes());

        for (ip, port) in &self.writers {
            buffer.extend_from_slice(ip); // Add 16-byte IP
            buffer.extend_from_slice(&port.to_le_bytes()); // Add 2-byte port in little-endian
        }
        Ok(buffer)
    }

    fn deserialize(buffer: &[u8]) -> Result<Self> {
        let mut writers = Vec::new();
        let mut offset = 0;
        while offset + 18 <= buffer.len() {
            let ip = <[u8; 16]>::try_from(&buffer[offset..offset + 16])
                .context("failed to get IP bytes")?;
            offset += 16;

            let port = u16::from_le_bytes(
                <[u8; 2]>::try_from(&buffer[offset..offset + 2])
                    .context("failed to get port bytes")?,
            );
            offset += 2;

            writers.push((ip, port));
        }

        Ok(ListWritersResponse { writers })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::Rng;

    #[test]
    fn test_roundtrip_basic() {
        let writers = vec![
            ([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1], 8080),
            (
                [32, 1, 13, 184, 0, 0, 0, 0, 0, 0, 0, 0, 255, 0, 66, 131],
                9090,
            ),
        ];

        let original = ListWritersResponse { writers };
        let serialized = original.serialize().unwrap();
        let deserialized = ListWritersResponse::deserialize(&serialized).unwrap();

        assert_eq!(original.writers, deserialized.writers);
    }

    #[test]
    fn test_roundtrip_random() {
        let mut rng = rand::thread_rng();
        let mut writers = Vec::new();

        for _ in 0..10 {
            let ip: [u8; 16] = rng.gen();
            let port: u16 = rng.gen();
            writers.push((ip, port));
        }

        let original = ListWritersResponse {
            writers: writers.clone(),
        };
        let serialized = original.serialize().unwrap();
        let deserialized = ListWritersResponse::deserialize(&serialized).unwrap();

        assert_eq!(original.writers, deserialized.writers);
    }
}
