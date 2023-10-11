use std::net::SocketAddr;

pub struct Config {
    pub(crate) socket_address: SocketAddr,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            socket_address: "127.0.0.1:3000".parse().unwrap(),
        }
    }
}
