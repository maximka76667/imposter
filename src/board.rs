use std::net::SocketAddr;

use tokio::sync::mpsc::{self, error::TrySendError};

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
        self.send(Command::SetPeriod(ms));
    }

    pub fn set_udp(&self, enabled: bool) {
        self.send(Command::SetUdp(enabled));
    }

    pub fn set_tcp(&self, enabled: bool) {
        self.send(Command::SetTcp(enabled));
    }

    pub fn set_mode(&self, mode: SimMode) {
        self.send(Command::SetMode(mode));
    }

    pub fn set_random_step(&self, step: f64) {
        self.send(Command::SetRandomStep(step));
    }

    fn send(&self, cmd: Command) {
        match self.ctrl_tx.try_send(cmd) {
            Ok(()) => {}
            Err(TrySendError::Full(_)) => {
                tracing::warn!(board = %self.name, "control channel full, command dropped");
            }
            Err(TrySendError::Closed(_)) => {
                tracing::error!(board = %self.name, "board task is dead");
            }
        }
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
