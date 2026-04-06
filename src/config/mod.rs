mod types;

pub use types::*;

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::Deserialize;

pub fn load(adj_dir: &Path) -> Result<Config> {
    let general_info: GeneralInfo = load_json(adj_dir.join("general_info.json"))?;

    let boards_map: HashMap<String, String> =
        load_json(adj_dir.join("boards.json")).context("loading boards.json")?;

    let mut boards = HashMap::new();

    for (name, rel_path) in &boards_map {
        let board_path = adj_dir.join(rel_path);
        let board_dir = board_path.parent().unwrap();
        let board_file: BoardFile =
            load_json(&board_path).with_context(|| format!("loading {}", board_path.display()))?;

        let mut measurements = Vec::new();
        for rel in &board_file.measurements {
            let path = board_dir.join(rel);
            let chunk: Vec<Measurement> =
                load_json(&path).with_context(|| format!("loading {}", path.display()))?;
            measurements.extend(chunk);
        }

        let mut packets = Vec::new();
        for rel in &board_file.packets {
            let path = board_dir.join(rel);
            let chunk: Vec<Packet> =
                load_json(&path).with_context(|| format!("loading {}", path.display()))?;
            packets.extend(chunk);
        }

        boards.insert(name.clone(), Board {
            board_id: board_file.board_id,
            board_ip: board_file.board_ip,
            measurements,
            packets,
        });
    }

    Ok(Config { general_info, boards })
}

fn load_json<T: for<'de> Deserialize<'de>>(path: impl Into<PathBuf>) -> Result<T> {
    let path = path.into();
    let text =
        std::fs::read_to_string(&path).with_context(|| format!("reading {}", path.display()))?;
    serde_json::from_str(&text).with_context(|| format!("parsing {}", path.display()))
}
