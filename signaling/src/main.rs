use matchbox_signaling::SignalingServer;
use std::net::{Ipv4Addr, SocketAddr};

#[tokio::main]
async fn main() {
    let port: u16 = std::env::args()
        .nth(1)
        .or_else(|| std::env::var("PORT").ok())
        .and_then(|p| p.parse().ok())
        .unwrap_or(3536);
    let addr: SocketAddr = (Ipv4Addr::UNSPECIFIED, port).into();
    println!("sinalização WebRTC ouvindo em ws://{addr}");
    let server = SignalingServer::full_mesh_builder(addr).build();
    server.serve().await.expect("servidor de sinalização falhou");
}
