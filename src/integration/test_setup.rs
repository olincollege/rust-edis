// kill any dangling info shard processes for test setup

#[cfg(test)]
async fn kill_dangling() {
    let system = sysinfo::System::new_all();

    let ps = system.processes().iter().filter(|(_, p)| {
        p.name().to_str().unwrap().eq("info")
            || p.name().to_str().unwrap().eq("read_shard")
            || p.name().to_str().unwrap().eq("write_shard")
            || p.name().to_str().unwrap().eq("client")
    });

    for (_, process) in ps {
        process.kill();
    }
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
}
#[cfg(test)]
pub async fn setup_test() {
    kill_dangling().await
 }

#[cfg(test)]
pub async fn test_teardown() {
    kill_dangling().await
}
