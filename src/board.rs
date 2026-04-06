use std::collections::HashMap;
use std::net::SocketAddr;

use tokio::sync::mpsc;

use crate::config::MeasurementType;
use crate::simulator;

pub enum Command {
    SetPeriod(u64),
    SetUdp(bool),
    SetTcp(bool),
}

pub struct BoardHandle {
    pub name: String,
    ctrl_tx: mpsc::Sender<Command>,
}

impl BoardHandle {
    pub fn set_period(&self, ms: u64) {
        let _ = self.ctrl_tx.try_send(Command::SetPeriod(ms));
    }

    pub fn set_udp(&self, enabled: bool) {
        let _ = self.ctrl_tx.try_send(Command::SetUdp(enabled));
    }

    pub fn set_tcp(&self, enabled: bool) {
        let _ = self.ctrl_tx.try_send(Command::SetTcp(enabled));
    }
}

pub fn spawn(
    name: String,
    board_ip: String,
    period_ms: u64,
    udp_enabled: bool,
    measurements: HashMap<String, MeasurementType>,
    data_packets: Vec<(u32, Vec<String>)>,
    dest: SocketAddr,
) -> BoardHandle {
    let (ctrl_tx, ctrl_rx) = mpsc::channel(16);

    tokio::spawn(simulator::run(
        name.clone(),
        board_ip,
        period_ms,
        udp_enabled,
        measurements,
        data_packets,
        dest,
        ctrl_rx,
    ));

    BoardHandle { name, ctrl_tx }
}
