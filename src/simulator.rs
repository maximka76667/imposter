use std::collections::HashMap;
use std::net::SocketAddr;
use std::time::Duration;

use tokio::sync::mpsc;
use tokio::time;

use crate::board::Command;
use crate::config::MeasurementType;
use crate::udp;

pub async fn run(
    name: String,
    board_ip: String,
    period_ms: u64,
    udp_enabled: bool,
    measurements: HashMap<String, MeasurementType>,
    data_packets: Vec<(u32, Vec<String>)>,
    dest: SocketAddr,
    mut ctrl_rx: mpsc::Receiver<Command>,
) {
    let mut period = Duration::from_millis(period_ms);

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
                if let Some(ref sock) = socket {
                    for (packet_id, variables) in &data_packets {
                        if let Err(e) = udp::send(sock, *packet_id, variables, &measurements, dest, &name).await {
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
                    Some(Command::SetTcp(_)) => {
                        // TCP not implemented yet
                    }
                    None => break,
                }
            }
        }
    }
}
