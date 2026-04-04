use std::collections::HashMap;
use std::net::{SocketAddr, UdpSocket};

use anyhow::{Context, Result};

use crate::config::{Board, Measurement, MeasurementType, Packet, PacketType};

pub fn bind(board: &Board) -> Result<UdpSocket> {
    let addr = format!("{}:0", board.board_ip);
    UdpSocket::bind(&addr).with_context(|| format!("binding UDP to {}", addr))
}

pub fn send(
    socket: &UdpSocket,
    packet: &Packet,
    measurements: &HashMap<&str, &Measurement>,
    dest: SocketAddr,
) -> Result<()> {
    let mut buf = Vec::new();

    buf.extend_from_slice(&(packet.id as u16).to_le_bytes());

    for var_id in &packet.variables {
        let measurement = measurements
            .get(var_id.as_str())
            .with_context(|| format!("unknown variable '{}'", var_id))?;
        write_zero(&mut buf, &measurement.kind);
    }

    socket
        .send_to(&buf, dest)
        .with_context(|| format!("sending packet {} to {}", packet.id, dest))?;
    tracing::debug!(packet_id = packet.id, bytes = buf.len(), dest = %dest, "sent");

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

pub fn measurement_map<'a>(board: &'a Board) -> HashMap<&'a str, &'a Measurement> {
    board
        .measurements
        .iter()
        .map(|m| (m.id.as_str(), m))
        .collect()
}

pub fn data_packets(board: &Board) -> impl Iterator<Item = &Packet> {
    board
        .packets
        .iter()
        .filter(|p| matches!(p.kind, PacketType::Data))
}
