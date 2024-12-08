use tokio::{io::AsyncWriteExt, net::tcp::OwnedWriteHalf};
use zerocopy::IntoBytes;
use anyhow::Result;

use crate::messages::message::MessagePayload;

pub async fn write_message(stream: &mut OwnedWriteHalf, message: Box<dyn MessagePayload>) -> Result<()> {
    let serialized = message.serialize()?;
    let serialized_buf = serialized.as_bytes();
    stream.write_all(serialized_buf).await?;
    Ok(())
}