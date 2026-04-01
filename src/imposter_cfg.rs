use std::collections::HashMap;
use std::path::Path;

use anyhow::{Context, Result};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct BoardCfg {
    pub period_ms: u64,
}

#[derive(Debug, Deserialize)]
pub struct ImposterCfg {
    pub default_period_ms: u64,
    #[serde(default)]
    pub boards: HashMap<String, BoardCfg>,
}

impl ImposterCfg {
    pub fn period_ms(&self, board_name: &str) -> u64 {
        self.boards
            .get(board_name)
            .map(|b| b.period_ms)
            .unwrap_or(self.default_period_ms)
    }
}

pub fn load(path: &Path) -> Result<ImposterCfg> {
    let text = std::fs::read_to_string(path)
        .with_context(|| format!("reading {}", path.display()))?;
    toml::from_str(&text).with_context(|| format!("parsing {}", path.display()))
}
