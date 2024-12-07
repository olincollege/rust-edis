
/* 
pub async fn write_message(mut stream: MuxStream<TcpStream>, message: Box<dyn MessagePayload>) -> Result<()> {
    let serialized = message.serialize()?;
    let mut serialized_buf = serialized.as_bytes();
    stream.write_all_buf(&mut serialized_buf).await?;
    Ok(())
}


mod tests {
    use super::*;
/* 
    #[tokio::test]  
    async fn test_read_message() {
        let stream = TcpStream::connect("127.0.0.1:8080").await.unwrap();
        let message = read_message(stream, true).await.unwrap();
        println!("{:?}", message);
    }
    */
}
*/