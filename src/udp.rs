use std::collections::HashMap;
use std::net::SocketAddr;

use anyhow::{Context, Result};
use tokio::net::UdpSocket;

use crate::config::{Board, MeasurementType, Packet, PacketType};

pub async fn bind(board_ip: &str) -> Result<UdpSocket> {
    let addr = format!("{}:0", board_ip);
    UdpSocket::bind(&addr)
        .await
        .with_context(|| format!("binding UDP to {}", addr))
}

pub async fn send(
    socket: &UdpSocket,
    packet_id: u32,
    variables: &[String],
    measurements: &HashMap<String, MeasurementType>,
    dest: SocketAddr,
    board: &str,
) -> Result<()> {
    let mut buf = Vec::new();

    buf.extend_from_slice(&(packet_id as u16).to_le_bytes());

    for var_id in variables {
        let kind = measurements
            .get(var_id.as_str())
            .with_context(|| format!("unknown variable '{}'", var_id))?;
        write_zero(&mut buf, kind);
    }

    socket
        .send_to(&buf, dest)
        .await
        .with_context(|| format!("sending packet {} to {}", packet_id, dest))?;
    tracing::debug!(board, packet_id, dest = %dest, "sent");

    Ok(())
}

fn write_zero(buf: &mut Vec<u8>, kind: &MeasurementType) {
    match kind {
        MeasurementType::Uint8
        | MeasurementType::Int8
        | MeasurementType::Bool
        | MeasurementType::Enum => {
            buf.push(0);
        }
        MeasurementType::Uint16 | MeasurementType::Int16 => {
            buf.extend_from_slice(&0u16.to_le_bytes());
        }
        MeasurementType::Uint32 | MeasurementType::Int32 | MeasurementType::Float32 => {
            buf.extend_from_slice(&0u32.to_le_bytes());
        }
        MeasurementType::Uint64 | MeasurementType::Int64 | MeasurementType::Float64 => {
            buf.extend_from_slice(&0u64.to_le_bytes());
        }
    }
}

pub fn measurement_map(board: &Board) -> HashMap<String, MeasurementType> {
    board
        .measurements
        .iter()
        .map(|m| (m.id.clone(), m.kind.clone()))
        .collect()
}

pub fn data_packets(board: &Board) -> impl Iterator<Item = &Packet> {
    board
        .packets
        .iter()
        .filter(|p| matches!(p.kind, PacketType::Data))
}
