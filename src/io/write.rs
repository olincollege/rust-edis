pub async fn write_message(mut stream: MuxStream<TcpStream>, message: Box<dyn MessagePayload>) -> Result<()> {
    let serialized = message.serialize()?;
    let mut serialized_buf = serialized.as_bytes();
    stream.write_all_buf(&mut serialized_buf).await?;
    Ok(())
}