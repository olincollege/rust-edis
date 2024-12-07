use anyhow::{Context, Result};
use int_enum::IntEnum;

#[repr(u8)]
#[derive(Debug, Clone, Copy, IntEnum, PartialEq, Eq)]
pub enum MessageType {
    Write = 0,              // 0 - third byte write request
    Read = 1,               // 1 - third byte read request
    GetClientShardInfo = 2, // 2 - get client shard info
    ReplicaInfo = 5,        // 5 - number of read/write replicas and other info
    QueryVersion = 6,       // 6 - query the latest version number
    GetVersion = 7,         // 7 - read key-value for a version number
}

pub trait MessagePayload {
    fn get_message_type(&self) -> MessageType;
    fn serialize(&self) -> Result<Vec<u8>>;
    fn deserialize(buffer: &[u8]) -> Result<Self>
    where
        Self: Sized;
}

/// Layout of the Message as described in architecture
/// | 4 bytes | 1 byte  | N bytes |
/// | totlen  | msgtype | payload |
/// Integers are always encoded in little-endian order
/// totlen includes the length of all fields (including itself)
pub struct Message<T: MessagePayload> {
    pub message_type: MessageType,
    pub message_payload: T,
}

impl<T: MessagePayload> Message<T> {
    pub fn serialize(&self) -> Result<Vec<u8>> {
        let mut buffer = Vec::new();

        let message_type: u8 = self.message_type as u8;
        let message_payload = self.message_payload.serialize()?;
        let total_length: u32 = 0;
        let total_length = (size_of_val(&total_length)
            + size_of_val(&message_type)
            + message_payload.len()) as u32;

        buffer.extend_from_slice(&total_length.to_le_bytes());
        buffer.extend_from_slice(&message_type.to_le_bytes());
        buffer.extend_from_slice(&message_payload);
        Ok(buffer)
    }

    pub fn deserialize(buffer: &[u8]) -> Result<Self> {
        let total_length: u32 = u32::from_le_bytes(
            buffer
                .get(0..4)
                .context("failed to get total length")?
                .try_into()?,
        );
        let message_type_bytes = u8::from_le_bytes(
            buffer
                .get(4..5)
                .context("failed to get message type")?
                .try_into()?,
        );
        let message_type = MessageType::try_from(message_type_bytes)
            .map_err(|_| anyhow::anyhow!("invalid message type"))?;
        let header_length = size_of_val(&total_length) + size_of_val(&message_type);
        let payload_length = (total_length as usize) - header_length;

        let message_payload = T::deserialize(
            buffer
                .get(header_length..header_length + payload_length)
                .context("failed to get message payload")?,
        )?;
        Ok(Message {
            message_type,
            message_payload,
        })
    }
}

mod tests {
    use super::*;
    use crate::messages::requests::write_request::WriteRequest;

    #[test]
    fn test_basic_roundtrip() {
        let message = Message {
            message_type: MessageType::Write,
            message_payload: WriteRequest {
                key: b"test".to_vec(),
                value: b"test".to_vec(),
            },
        };
        let serialized = message.serialize().unwrap();
        let deserialized = Message::<WriteRequest>::deserialize(&serialized).unwrap();
        assert_eq!(message.message_type, deserialized.message_type);
        assert_eq!(
            message.message_payload.key,
            deserialized.message_payload.key
        );
        assert_eq!(
            message.message_payload.value,
            deserialized.message_payload.value
        );
    }
}
