use std::collections::HashMap;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct GeneralInfo {
    pub ports: HashMap<String, u16>,
    pub addresses: HashMap<String, String>,
    pub units: HashMap<String, String>,
    pub message_ids: HashMap<String, u32>,
}

#[derive(Debug, Deserialize)]
pub struct BoardFile {
    pub board_id: u32,
    pub board_ip: String,
    pub measurements: Vec<String>,
    pub packets: Vec<String>,
    #[serde(default)]
    pub sockets: Vec<String>, // parsed, ignored
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MeasurementType {
    Uint8,
    Uint16,
    Uint32,
    Uint64,
    Int8,
    Int16,
    Int32,
    Int64,
    Float32,
    Float64,
    Enum,
    Bool,
}

#[derive(Debug, Deserialize)]
pub struct Measurement {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub kind: MeasurementType,
    #[serde(rename = "enumValues")]
    pub enum_values: Option<Vec<String>>,
    #[serde(rename = "safeRange")]
    pub safe_range: Option<[f64; 2]>,
    #[serde(rename = "warningRange")]
    pub warning_range: Option<[f64; 2]>,
    #[serde(rename = "podUnits")]
    pub pod_units: Option<String>,
    #[serde(rename = "displayUnits")]
    pub display_units: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PacketType {
    Data,
    Order,
}

#[derive(Debug, Deserialize)]
pub struct Packet {
    pub id: u32,
    #[serde(rename = "type")]
    pub kind: PacketType,
    pub name: String,
    #[serde(default)]
    pub variables: Vec<String>,
}

#[derive(Debug)]
pub struct Board {
    pub board_id: u32,
    pub board_ip: String,
    pub measurements: Vec<Measurement>,
    pub packets: Vec<Packet>,
}

#[derive(Debug)]
pub struct Config {
    pub general_info: GeneralInfo,
    pub boards: HashMap<String, Board>,
}

pub fn load(adj_dir: &Path) -> Result<Config> {
    let general_info: GeneralInfo = load_json(adj_dir.join("general_info.json"))?;

    let boards_map: HashMap<String, String> =
        load_json(adj_dir.join("boards.json")).context("loading boards.json")?;

    let mut boards = HashMap::new();

    // Could be made parallel if necessary
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

        boards.insert(
            name.clone(),
            Board {
                board_id: board_file.board_id,
                board_ip: board_file.board_ip,
                measurements,
                packets,
            },
        );
    }

    Ok(Config {
        general_info,
        boards,
    })
}

fn load_json<T: for<'de> Deserialize<'de>>(path: impl Into<PathBuf>) -> Result<T> {
    let path = path.into();
    let text =
        std::fs::read_to_string(&path).with_context(|| format!("reading {}", path.display()))?;
    serde_json::from_str(&text).with_context(|| format!("parsing {}", path.display()))
}
