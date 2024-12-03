#[derive(Debug)]
enum MessageType {
    Write = 0,  // 0 - third byte write request
    Read = 1,   // 1 - third byte read request
    ListWrites = 2, // 2 - list of ip/ports of write replicas
    ListReaders = 3,  // 3 - list of ip/ports of read replicas
    ReplicaInfo = 4,   // 4 - number of read/write replicas and other info
    QueryVersion = 5,  // 5 - query the latest version number
    ReadByVersion = 6, // 6 - read key-value for a version number
}

#[derive(Debug)]
struct Message {
    msg_type: MessageType,
    payload: Vec<u8>,
}

impl Message {
    fn new(message_type: MessageType, payload: Vec<u8>) -> Self {
        todo!()
    }

    fn encode(&self) -> Vec<u8> {
        todo!()
    }

    fn decode(buffer: &[u8]) -> Option<Self> {
        todo!()
    }
}
