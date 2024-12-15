use anyhow::Result;
use tokio::{io::AsyncWriteExt, net::tcp::OwnedWriteHalf};
use zerocopy::IntoBytes;

use crate::messages::message::{Message, MessagePayload};

pub async fn write_message<T: MessagePayload>(
    stream: &mut OwnedWriteHalf,
    message: Message<T>,
) -> Result<()> {
    let serialized = message.serialize()?;
    let serialized_buf = serialized.as_bytes();
    stream.write_all(serialized_buf).await?;
    stream.flush().await?;
    // println!("finished writing {written} bytes!");
    Ok(())
}
