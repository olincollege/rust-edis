
#[cfg(test)]
mod tests {
    use anyhow::Result;
    use assert_cmd::prelude::*;
    use predicates::prelude::*;
    use serial_test::serial;
    use std::path::{absolute, Path};
    use std::process::Command;
    use port_killer::{kill, kill_by_pids};

    use crate::messages::requests::get_client_shard_info_request::GetClientShardInfoRequest;
    use crate::messages::responses::get_client_shard_info_response;
    use crate::utils::constants::MAIN_INSTANCE_IP_PORT;
    use crate::utils::test_client;

    #[serial]
    #[tokio::test]
    async fn test_basic_integration() -> Result<()> {
        // kill any dangling process
        kill(8080)?;

        // start everything 
        let mut info_cmd = Command::cargo_bin("info")?;
        let mut read_cmd = Command::cargo_bin("read_shard")?;
        let mut write_cmd = Command::cargo_bin("write_shard")?;

        info_cmd.arg("--write-shards=1").spawn()?;
        read_cmd.spawn()?;
        write_cmd.spawn()?;
        tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;

        // get the client shard into list
        let test_client = test_client::TestRouterClient::new();
        test_client.get_client().queue_request(GetClientShardInfoRequest {}, MAIN_INSTANCE_IP_PORT).await?;
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

        let client_shard_info_responses = test_client.get_client_shard_info_responses.lock().unwrap();
        assert_eq!(client_shard_info_responses.len(), 1);
        assert_eq!(client_shard_info_responses[0].num_write_shards, 1);

        Ok(())
    }

}