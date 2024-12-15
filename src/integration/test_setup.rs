// kill any dangling info shard processes for test setup
pub fn setup_test() {
    let system = sysinfo::System::new_all();

    let ps = system.processes().iter().filter(|(_, p)| {
        p.name().to_str().unwrap().eq("info")
            || p.name().to_str().unwrap().eq("read_shard")
            || p.name().to_str().unwrap().eq("write_shard")
    });

    for (_, process) in ps {
        process.kill();
    }
}

pub fn test_teardown() {
    let system = sysinfo::System::new_all();

    let ps = system.processes().iter().filter(|(_, p)| {
        p.name().to_str().unwrap().eq("info")
            || p.name().to_str().unwrap().eq("read_shard")
            || p.name().to_str().unwrap().eq("write_shard")
    });

    for (_pid, process) in ps {
        let name = process.name().to_str().unwrap();
        println!("killing {name}");
        process.kill();
    }
}
