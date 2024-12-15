use std::net::{Ipv6Addr, SocketAddrV6};

pub static MAIN_INSTANCE_IP_PORT: SocketAddrV6 = SocketAddrV6::new(Ipv6Addr::LOCALHOST, 8080, 0, 0);
