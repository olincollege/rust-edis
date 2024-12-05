use crate::messages::message::{MessagePayload, MessageType};
use anyhow::{Context, Result};

pub struct ListReadersResponse {
    pub count: u16,                    // Number of readers
    pub readers: Vec<([u8; 16], u16)>, // Each reader is a tuple of 16-byte IP and 2-byte port
}

/// Layout of the ListReadersResponse
/// | 2 bytes (reader count) | 18 bytes per reader | N readers |
/// |       Reader Count      |     IP (16 bytes)   | Port (2 bytes) |
impl MessagePayload for ListReadersResponse {
    fn get_message_type(&self) -> MessageType {
        MessageType::ListReaders
    }

    fn serialize(&self) -> Result<Vec<u8>> {
        let mut buffer = Vec::new();

        // Add the number of readers (2 bytes, little-endian)
        buffer.extend_from_slice(&self.count.to_le_bytes());

        // Serialize each reader
        for (ip, port) in &self.readers {
            buffer.extend_from_slice(ip); // Add 16-byte IP
            buffer.extend_from_slice(&port.to_le_bytes()); // Add 2-byte port in little-endian
        }

        Ok(buffer)
    }

    fn deserialize(buffer: &[u8]) -> Result<Self> {
        let mut offset = 0;

        // Read the number of readers (2 bytes, little-endian)
        let count = u16::from_le_bytes(
            <[u8; 2]>::try_from(&buffer[offset..offset + 2])
                .context("failed to get reader count")?,
        );
        offset += 2;

        let mut readers = Vec::new();

        // Deserialize each reader
        for _ in 0..count {
            let ip = <[u8; 16]>::try_from(&buffer[offset..offset + 16])
                .context("failed to get IP bytes")?;
            offset += 16;

            let port = u16::from_le_bytes(
                <[u8; 2]>::try_from(&buffer[offset..offset + 2])
                    .context("failed to get port bytes")?,
            );
            offset += 2;

            readers.push((ip, port));
        }

        Ok(ListReadersResponse { count, readers })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::Rng;

    #[test]
    fn test_roundtrip_basic() {
        let readers = vec![
            ([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1], 8080),
            (
                [32, 1, 13, 184, 0, 0, 0, 0, 0, 0, 0, 0, 255, 0, 66, 131],
                9090,
            ),
        ];

        let original = ListReadersResponse {
            count: readers.len() as u16,
            readers,
        };
        let serialized = original.serialize().unwrap();
        let deserialized = ListReadersResponse::deserialize(&serialized).unwrap();

        assert_eq!(original.count, deserialized.count);
        assert_eq!(original.readers, deserialized.readers);
    }

    #[test]
    fn test_roundtrip_random() {
        let mut rng = rand::thread_rng();
        let mut readers = Vec::new();

        for _ in 0..10 {
            let ip: [u8; 16] = rng.gen();
            let port: u16 = rng.gen();
            readers.push((ip, port));
        }

        let original = ListReadersResponse {
            count: readers.len() as u16,
            readers: readers.clone(),
        };
        let serialized = original.serialize().unwrap();
        let deserialized = ListReadersResponse::deserialize(&serialized).unwrap();

        assert_eq!(original.count, deserialized.count);
        assert_eq!(original.readers, deserialized.readers);
    }
}
