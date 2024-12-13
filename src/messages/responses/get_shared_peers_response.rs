use crate::messages::message::{MessagePayload, MessageType};
use anyhow::{Context, Result};

pub struct GetSharedPeersResponse {
    pub peer_ips: Vec<(u128, u16)>,
}

/// Layout of the GetSharedPeersResponse
/// | 18 bytes | (N - 1) * (18 bytes) |
/// | IP/Port of writer | IP/Port of reader peers ... |
impl MessagePayload for GetSharedPeersResponse {
    fn get_message_type(&self) -> MessageType {
        MessageType::GetSharedPeers
    }

    fn is_request(&self) -> bool {
        false
    }

    fn serialize(&self) -> Result<Vec<u8>> {
        let mut buffer = Vec::new();

        // Add Peer IPs
        for (ip, port) in &self.peer_ips {
            buffer.extend_from_slice(&ip.to_le_bytes());
            buffer.extend_from_slice(&port.to_le_bytes());
        }
        Ok(buffer)
    }

    fn deserialize(buffer: &[u8]) -> Result<Self> {
        let mut offset = 0;

        let mut peer_ips = Vec::new();
        while offset < buffer.len() {
            // Read IP (16 bytes)
            let ip = <[u8; 16]>::try_from(&buffer[offset..offset + 16])
                .context("failed to get IP bytes")?;
            let ip = u128::from_le_bytes(ip);
            offset += 16;

            // Read port (2 bytes, little-endian)
            let port = u16::from_le_bytes(
                <[u8; 2]>::try_from(&buffer[offset..offset + 2])
                    .context("failed to get port bytes")?,
            );
            offset += 2;

            peer_ips.push((ip, port));
        }

        Ok(GetSharedPeersResponse { peer_ips })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::Rng;

    #[test]
    fn test_roundtrip_basic() {
        let original = GetSharedPeersResponse {
            peer_ips: vec![(1, 8080), (1, 8081)],
        };
        let serialized = original.serialize().unwrap();
        let deserialized = GetSharedPeersResponse::deserialize(&serialized).unwrap();
        assert_eq!(original.peer_ips, deserialized.peer_ips);
        assert_eq!(original.peer_ips, vec![(1, 8080,), (1, 8081,),])
    }

    #[test]
    fn test_roundtrip_random() {
        for _ in 0..1000 {
            let mut rng = rand::thread_rng();
            let ip: u128 = rng.gen();
            let port: u16 = rng.gen();
            let original = GetSharedPeersResponse {
                peer_ips: vec![(ip, port)],
            };
            let serialized = original.serialize().unwrap();
            let deserialized = GetSharedPeersResponse::deserialize(&serialized).unwrap();
            assert_eq!(original.peer_ips, deserialized.peer_ips);
        }
    }
}
