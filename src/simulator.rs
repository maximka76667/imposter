use std::net::SocketAddr;
use std::time::Duration;

use tokio::sync::mpsc;
use tokio::time;

use crate::board::Command;
use crate::config::MeasurementType;
use crate::state::{MeasurementSpec, MeasurementState, SimMode};
use crate::udp;

pub async fn run(
    name: String,
    board_ip: String,
    period_ms: u64,
    udp_enabled: bool,
    mut specs: Vec<MeasurementSpec>,
    data_packets: Vec<(u32, Vec<String>)>,
    dest: SocketAddr,
    mut mode: SimMode,
    mut ctrl_rx: mpsc::Receiver<Command>,
) {
    let mut period = Duration::from_millis(period_ms);
    let mut state = MeasurementState::new(&specs, &mode);

    let mut socket = if udp_enabled {
        match udp::bind(&board_ip).await {
            Ok(s) => {
                tracing::info!(board = %name, addr = %board_ip, packets = data_packets.len(), "UDP started");
                Some(s)
            }
            Err(e) => {
                tracing::error!(board = %name, err = %e, "failed to bind UDP");
                None
            }
        }
    } else {
        None
    };

    let mut interval = time::interval(period);
    interval.set_missed_tick_behavior(time::MissedTickBehavior::Skip);

    loop {
        tokio::select! {
            _ = interval.tick() => {
                state.tick(&specs, &mode);
                if let Some(ref sock) = socket {
                    for (packet_id, variables) in &data_packets {
                        let buf = build_packet(*packet_id, variables, &state);
                        if let Err(e) = udp::send(sock, &buf, dest, &name, *packet_id).await {
                            tracing::error!(board = %name, err = %e, "UDP send failed");
                        }
                    }
                }
            }
            cmd = ctrl_rx.recv() => {
                match cmd {
                    Some(Command::SetPeriod(ms)) => {
                        period = Duration::from_millis(ms);
                        interval = time::interval(period);
                        interval.set_missed_tick_behavior(time::MissedTickBehavior::Skip);
                        tracing::info!(board = %name, period_ms = ms, "period updated");
                    }
                    Some(Command::SetUdp(enabled)) => {
                        if enabled && socket.is_none() {
                            socket = match udp::bind(&board_ip).await {
                                Ok(s) => {
                                    tracing::info!(board = %name, addr = %board_ip, packets = data_packets.len(), "UDP started");
                                    Some(s)
                                }
                                Err(e) => {
                                    tracing::error!(board = %name, err = %e, "failed to bind UDP");
                                    None
                                }
                            };
                        } else if !enabled && socket.is_some() {
                            socket = None;
                            tracing::info!(board = %name, "UDP stopped");
                        }
                    }
                    Some(Command::SetMode(new_mode)) => {
                        mode = new_mode;
                        state = MeasurementState::new(&specs, &mode);
                        tracing::info!(board = %name, mode = ?mode, "mode updated");
                    }
                    Some(Command::SetRandomStep(step)) => {
                        for spec in specs.iter_mut() {
                            spec.random_step = step;
                        }
                        tracing::info!(board = %name, step, "random_step updated");
                    }
                    Some(Command::SetTcp(_)) => {
                        // TCP not implemented yet
                    }
                    None => break,
                }
            }
        }
    }
}

fn build_packet(
    packet_id: u32,
    variables: &[String],
    state: &MeasurementState,
) -> Vec<u8> {
    let mut buf = Vec::new();
    buf.extend_from_slice(&(packet_id as u16).to_le_bytes());
    for var_id in variables {
        if let Some((value, kind)) = state.get(var_id) {
            tracing::debug!(packet_id, var = var_id, value, "tick");
            write_value(&mut buf, kind, value);
        }
    }
    buf
}

fn write_value(buf: &mut Vec<u8>, kind: &MeasurementType, value: f64) {
    match kind {
        MeasurementType::Enum => buf.push(value as u8),
        MeasurementType::Bool => buf.push(if value != 0.0 { 1 } else { 0 }),
        MeasurementType::Uint8 => buf.push(value as u8),
        MeasurementType::Int8 => buf.push(value as i8 as u8),
        MeasurementType::Uint16 => buf.extend_from_slice(&(value as u16).to_le_bytes()),
        MeasurementType::Int16 => buf.extend_from_slice(&(value as i16).to_le_bytes()),
        MeasurementType::Uint32 => buf.extend_from_slice(&(value as u32).to_le_bytes()),
        MeasurementType::Int32 => buf.extend_from_slice(&(value as i32).to_le_bytes()),
        MeasurementType::Float32 => buf.extend_from_slice(&(value as f32).to_le_bytes()),
        MeasurementType::Uint64 => buf.extend_from_slice(&(value as u64).to_le_bytes()),
        MeasurementType::Int64 => buf.extend_from_slice(&(value as i64).to_le_bytes()),
        MeasurementType::Float64 => buf.extend_from_slice(&value.to_le_bytes()),
    }
}
