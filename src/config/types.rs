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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_float_measurement() {
        let m: Measurement = serde_json::from_str(r#"{
            "id": "brake_pressure",
            "name": "Brake Pressure",
            "type": "float32",
            "safeRange": [0.0, 100.0]
        }"#).unwrap();
        assert_eq!(m.id, "brake_pressure");
        assert!(matches!(m.kind, MeasurementType::Float32));
        assert_eq!(m.safe_range, Some([0.0, 100.0]));
        assert!(m.warning_range.is_none());
    }

    #[test]
    fn parse_enum_measurement() {
        let m: Measurement = serde_json::from_str(r#"{
            "id": "state",
            "name": "State",
            "type": "enum",
            "enumValues": ["idle", "running", "error"]
        }"#).unwrap();
        assert!(matches!(m.kind, MeasurementType::Enum));
        assert_eq!(m.enum_values.unwrap().len(), 3);
    }

    #[test]
    fn parse_data_packet() {
        let p: Packet = serde_json::from_str(r#"{
            "id": 249,
            "type": "data",
            "name": "Current State",
            "variables": ["brake_pressure", "state"]
        }"#).unwrap();
        assert_eq!(p.id, 249);
        assert!(matches!(p.kind, PacketType::Data));
        assert_eq!(p.variables.len(), 2);
    }

    #[test]
    fn parse_order_packet_empty_variables() {
        let p: Packet = serde_json::from_str(r#"{
            "id": 502,
            "type": "order",
            "name": "Turn on PFM"
        }"#).unwrap();
        assert!(matches!(p.kind, PacketType::Order));
        assert!(p.variables.is_empty());
    }
}
