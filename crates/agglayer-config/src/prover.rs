use std::net::{IpAddr, Ipv4Addr, SocketAddr};
const fn default_socket_addr() -> SocketAddr {
    SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080)
}

pub(crate) fn default_prover_entrypoint() -> String {
    format!("http://{}", default_socket_addr())
}
