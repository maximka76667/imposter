use std::collections::HashMap;
use std::path::Path;

use anyhow::{Context, Result};
use serde::Deserialize;

use crate::state::SimMode;

fn default_true() -> bool {
    true
}

fn default_random_step() -> f64 {
    0.02
}

#[derive(Debug, Deserialize)]
pub struct BoardCfg {
    pub period_ms: Option<u64>,
    #[serde(rename = "enable_udp")]
    pub udp: Option<bool>,
    #[serde(rename = "enable_tcp")]
    pub tcp: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct ImposterCfg {
    pub default_period_ms: u64,
    #[serde(rename = "enable_udp")]
    pub udp: Option<bool>,
    #[serde(rename = "enable_tcp")]
    pub tcp: Option<bool>,
    #[serde(default = "default_true", rename = "default_enable_udp")]
    pub default_udp: bool,
    #[serde(default = "default_true", rename = "default_enable_tcp")]
    pub default_tcp: bool,
    #[serde(default)]
    pub verbose: bool,
    #[serde(default)]
    pub mode: SimMode,
    #[serde(default = "default_random_step")]
    pub random_step: f64,
    #[serde(default)]
    pub boards: HashMap<String, BoardCfg>,
}

impl ImposterCfg {
    pub fn period_ms(&self, board_name: &str) -> u64 {
        self.boards
            .get(board_name)
            .and_then(|b| b.period_ms)
            .unwrap_or(self.default_period_ms)
    }

    pub fn udp_enabled(&self, board_name: &str) -> bool {
        if let Some(global) = self.udp {
            return global;
        }
        self.boards
            .get(board_name)
            .and_then(|b| b.udp)
            .unwrap_or(self.default_udp)
    }

    pub fn tcp_enabled(&self, board_name: &str) -> bool {
        if let Some(global) = self.tcp {
            return global;
        }
        self.boards
            .get(board_name)
            .and_then(|b| b.tcp)
            .unwrap_or(self.default_tcp)
    }
}

pub fn load(path: &Path) -> Result<ImposterCfg> {
    let text =
        std::fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
    toml::from_str(&text).with_context(|| format!("parsing {}", path.display()))
}
