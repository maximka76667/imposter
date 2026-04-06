use std::net::SocketAddr;

use tokio::sync::mpsc;

use crate::simulator;
use crate::state::{MeasurementSpec, SimMode};

pub enum Command {
    SetPeriod(u64),
    SetUdp(bool),
    SetTcp(bool),
    SetMode(SimMode),
    SetRandomStep(f64),
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

    pub fn set_mode(&self, mode: SimMode) {
        let _ = self.ctrl_tx.try_send(Command::SetMode(mode));
    }

    pub fn set_random_step(&self, step: f64) {
        let _ = self.ctrl_tx.try_send(Command::SetRandomStep(step));
    }
}

pub fn spawn(
    name: String,
    board_ip: String,
    period_ms: u64,
    udp_enabled: bool,
    specs: Vec<MeasurementSpec>,
    data_packets: Vec<(u32, Vec<String>)>,
    dest: SocketAddr,
    mode: SimMode,
) -> BoardHandle {
    let (ctrl_tx, ctrl_rx) = mpsc::channel(16);

    tokio::spawn(simulator::run(
        name.clone(),
        board_ip,
        period_ms,
        udp_enabled,
        specs,
        data_packets,
        dest,
        mode,
        ctrl_rx,
    ));

    BoardHandle { name, ctrl_tx }
}
