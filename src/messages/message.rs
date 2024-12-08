use anyhow::{Context, Result};
use int_enum::IntEnum;
use std::any::Any;

use super::requests::{
    get_client_shard_info_request::GetClientShardInfoRequest,
    query_version_request::QueryVersionRequest, read_request::ReadRequest,
    write_request::WriteRequest,
};

use super::responses::read_response::ReadResponse;
use super::responses::write_response::WriteResponse;
use super::responses::{
    get_client_shard_info_response::GetClientShardInfoResponse,
    query_version_response::QueryVersionResponse,
};

#[repr(u8)]
#[derive(Debug, Clone, Copy, IntEnum, PartialEq, Eq)]
pub enum MessageType {
    Write = 0,              // 0 - third byte write request
    Read = 1,               // 1 - third byte read request
    GetClientShardInfo = 2, // 2 - get client shard info
    QueryVersion = 3,       // 3 - query the latest version number
    GetVersion = 4,         // 4 - read key-value for a version number
    AnnounceShard = 5,      // 5 - announce a shard
    GetSharedPeers = 6,     // 6 - get shared peers
}

pub trait MessagePayload: AsAny + Send {
    fn is_request(&self) -> bool;
    fn get_message_type(&self) -> MessageType;
    fn serialize(&self) -> Result<Vec<u8>>;
    fn deserialize(buffer: &[u8]) -> Result<Self>
    where
        Self: Sized;
}
pub trait AsAny: Any {
    fn as_any(&self) -> &dyn Any;
}
impl<T: Any> AsAny for T {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

pub fn bytes_as_message(buffer: &[u8]) -> Result<Box<dyn MessagePayload>> {
    let message_type =
        MessageType::try_from(buffer[4]).map_err(|_| anyhow::anyhow!("invalid message type"))?;
    let is_request = buffer[5] == 1;
    let result: Box<dyn MessagePayload> = match message_type {
        MessageType::Write => {
            match is_request {
                true => Box::new(Message::<WriteRequest>::deserialize(buffer)?.message_payload),
                false => Box::new(Message::<WriteResponse>::deserialize(buffer)?.message_payload),
            }
        }
        MessageType::Read => {
            match is_request {
                true => Box::new(Message::<ReadRequest>::deserialize(buffer)?.message_payload),
                false => Box::new(Message::<ReadResponse>::deserialize(buffer)?.message_payload),
            }
        }
        MessageType::GetClientShardInfo => {
            match is_request {
                true => Box::new(Message::<GetClientShardInfoRequest>::deserialize(buffer)?.message_payload),
                false => Box::new(Message::<GetClientShardInfoResponse>::deserialize(buffer)?.message_payload),
            }
        }
        MessageType::QueryVersion => {
            match is_request {
                true => Box::new(Message::<QueryVersionRequest>::deserialize(buffer)?.message_payload),
                false => Box::new(Message::<QueryVersionResponse>::deserialize(buffer)?.message_payload),
            }
        }
        _ => {
            println!("failed to parse");
            return Err(anyhow::anyhow!("unsupported message type"));
        }
    };
    Ok(result)
}

/// Layout of the Message as described in architecture
/// | 4 bytes | 1 byte  | 1 byte  | N bytes |
/// | totlen  | msgtype | is_req  | payload |
/// Integers are always encoded in little-endian order
/// totlen includes the length of all fields (including itself)
/// is_req is 1 for requests, 0 for responses
pub struct Message<T: MessagePayload> {
    pub is_request: bool,
    pub message_type: MessageType,
    pub message_payload: T,
}

impl<T: MessagePayload> Message<T> {
    pub fn serialize(&self) -> Result<Vec<u8>> {
        let mut buffer = Vec::new();

        let message_type: u8 = self.message_type as u8;
        let is_request: u8 = if self.is_request { 1 } else { 0 };
        let message_payload = self.message_payload.serialize()?;
        let total_length: u32 = 0;
        let total_length = (size_of_val(&total_length)
            + size_of_val(&message_type)
            + size_of_val(&is_request)
            + message_payload.len()) as u32;

        buffer.extend_from_slice(&total_length.to_le_bytes());
        buffer.extend_from_slice(&message_type.to_le_bytes());
        buffer.extend_from_slice(&is_request.to_le_bytes());
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
        let is_request = u8::from_le_bytes(
            buffer
                .get(5..6)
                .context("failed to get is_request")?
                .try_into()?,
        ) == 1;
        let message_type = MessageType::try_from(message_type_bytes)
            .map_err(|_| anyhow::anyhow!("invalid message type"))?;
        let header_length = size_of_val(&total_length) + size_of_val(&message_type) + 1;
        let payload_length = (total_length as usize) - header_length;

        let message_payload = T::deserialize(
            buffer
                .get(header_length..header_length + payload_length)
                .context("failed to get message payload")?,
        )?;
        Ok(Message {
            is_request,
            message_type,
            message_payload,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::messages::requests::write_request::WriteRequest;

    #[test]
    fn test_basic_roundtrip() {
        let message = Message {
            is_request: true,
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
        assert_eq!(message.is_request, deserialized.is_request);
    }
}
