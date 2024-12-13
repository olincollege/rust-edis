use crate::messages::message::{MessagePayload, MessageType};
use anyhow::{Context, Result};

/// Layout of the GetClientShardInfoResponse as described in architecture
/// | 2 bytes | N * 18 bytes | M * 18 bytes |
/// | num_write_shards | write_shard_info | read_shard_info |
pub struct GetClientShardInfoResponse {
    pub num_write_shards: u16,
    pub write_shard_info: Vec<(u128, u16)>,
    pub read_shard_info: Vec<(u128, u16)>,
}

impl MessagePayload for GetClientShardInfoResponse {
    fn get_message_type(&self) -> MessageType {
        MessageType::GetClientShardInfo
    }

    fn is_request(&self) -> bool {
        false
    }

    fn serialize(&self) -> Result<Vec<u8>> {
        let mut buffer = Vec::new();
        buffer.extend_from_slice(&self.num_write_shards.to_le_bytes());
        for (ip, port) in &self.write_shard_info {
            buffer.extend_from_slice(&ip.to_le_bytes());
            buffer.extend_from_slice(&port.to_le_bytes());
        }
        for (ip, port) in &self.read_shard_info {
            buffer.extend_from_slice(&ip.to_le_bytes());
            buffer.extend_from_slice(&port.to_le_bytes());
        }
        Ok(buffer)
    }

    fn deserialize(buffer: &[u8]) -> Result<Self> {
        let num_write_shards = u16::from_le_bytes(
            buffer
                .get(0..2)
                .context("failed to get num_write_shards")?
                .try_into()?,
        );

        let mut write_shard_info = Vec::new();
        let mut read_shard_info = Vec::new();

        for i in 0..(num_write_shards as usize) {
            let write_shard_offset: usize = 2;

            let ip_write = u128::from_le_bytes(buffer
                .get((write_shard_offset + i * 18)..write_shard_offset + i * 18 + 16)
                .context("failed to get write shard ip")?
                .try_into()?);
            let port_write = u16::from_le_bytes(
                buffer
                    .get(write_shard_offset + i * 18 + 16..write_shard_offset + i * 18 + 18)
                    .context("failed to get write shard port")?
                    .try_into()?,
            );

            let read_shard_offset: usize = write_shard_offset + 18 * num_write_shards as usize;
            let ip_read = u128::from_le_bytes(buffer
                .get(read_shard_offset + i * 18..read_shard_offset + i * 18 + 16)
                .context("failed to get read shard ip")?
                .try_into()?);
            let port_read = u16::from_le_bytes(
                buffer
                    .get(read_shard_offset + i * 18 + 16..read_shard_offset + i * 18 + 18)
                    .context("failed to get read shard port")?
                    .try_into()?,
            );

            write_shard_info.push((ip_write, port_write));
            read_shard_info.push((ip_read, port_read));
        }

        Ok(GetClientShardInfoResponse {
            num_write_shards,
            write_shard_info,
            read_shard_info,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::Rng;
    #[test]
    fn test_roundtrip_basic() {
        let original = GetClientShardInfoResponse {
            num_write_shards: 2,
            write_shard_info: vec![
                (
                    12121,
                    8080,
                ),
                (
                    321093201,
                    8081,
                ),
            ],
            read_shard_info: vec![
                (
                    321098,
                    9080,
                ),
                (
                    909032190,
                    9081,
                ),
            ],
        };
        let serialized = original.serialize().unwrap();
        let deserialized = GetClientShardInfoResponse::deserialize(&serialized).unwrap();
        assert_eq!(original.num_write_shards, deserialized.num_write_shards);
        assert_eq!(original.write_shard_info, deserialized.write_shard_info);
        assert_eq!(original.read_shard_info, deserialized.read_shard_info);
    }

    #[test]
    fn test_roundtrip_random() {
        for _ in 0..1000 {
            let mut rng = rand::thread_rng();
            let num_shards = rng.gen_range(1..11);

            let mut write_shard_info = Vec::new();
            let mut read_shard_info = Vec::new();

            for _ in 0..num_shards {
                let write_ip: u128 = rng.gen();
                let write_port = rng.gen();
                write_shard_info.push((write_ip, write_port));

                let read_ip: u128 = rng.gen();
                let read_port = rng.gen();
                read_shard_info.push((read_ip, read_port));
            }

            let original = GetClientShardInfoResponse {
                num_write_shards: num_shards as u16,
                write_shard_info,
                read_shard_info,
            };
            let serialized = original.serialize().unwrap();
            let deserialized = GetClientShardInfoResponse::deserialize(&serialized).unwrap();

            assert_eq!(original.num_write_shards, deserialized.num_write_shards);
            assert_eq!(original.write_shard_info, deserialized.write_shard_info);
            assert_eq!(original.read_shard_info, deserialized.read_shard_info);
        }
    }
}
