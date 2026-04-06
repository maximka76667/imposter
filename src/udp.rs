use std::net::SocketAddr;

use anyhow::{Context, Result};
use tokio::net::UdpSocket;

pub async fn bind(board_ip: &str) -> Result<UdpSocket> {
    let addr = format!("{}:0", board_ip);
    UdpSocket::bind(&addr)
        .await
        .with_context(|| format!("binding UDP to {}", addr))
}

pub async fn send(
    socket: &UdpSocket,
    buf: &[u8],
    dest: SocketAddr,
    board: &str,
    packet_id: u32,
) -> Result<()> {
    socket
        .send_to(buf, dest)
        .await
        .with_context(|| format!("sending packet {} to {}", packet_id, dest))?;
    tracing::debug!(board, packet_id, dest = %dest, "sent");
    Ok(())
}
