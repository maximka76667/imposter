use std::collections::HashMap;

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

#[derive(Debug, Clone, Deserialize)]
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
