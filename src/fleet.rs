use std::net::SocketAddr;

use anyhow::Context;

use crate::board::{self, BoardHandle};
use crate::config::{Board, Config, Packet, PacketType};
use crate::imposter_cfg::ImposterCfg;
use crate::state::MeasurementSpec;

pub fn launch(config: &Config, imposter_cfg: &ImposterCfg) -> anyhow::Result<Vec<BoardHandle>> {
    for name in imposter_cfg.boards.keys() {
        if !config.boards.contains_key(name.as_str()) {
            tracing::warn!(board = %name, "unknown board in imposter.toml, ignoring");
        }
    }

    let mut names: Vec<&str> = config.boards.keys().map(String::as_str).collect();
    names.sort_unstable_by_key(|name| {
        config.boards[*name]
            .board_ip
            .rsplit('.')
            .next()
            .and_then(|s| s.parse::<u8>().ok())
            .unwrap_or(0)
    });

    let dest = backend_dest(config)?;

    for name in &names {
        let board = &config.boards[*name];
        let counts = if imposter_cfg.verbose {
            format!(" | msrmnts:{} | pkts:{}", board.measurements.len(), board.packets.len())
        } else {
            String::new()
        };
        tracing::info!(
            "> {} » {} | id:{} | {}ms | udp:{} | tcp:{}{}",
            board.board_ip,
            name,
            board.board_id,
            imposter_cfg.period_ms(name),
            imposter_cfg.udp_enabled(name),
            imposter_cfg.tcp_enabled(name),
            counts,
        );
    }

    let mut handles = Vec::new();

    for name in &names {
        let board = &config.boards[*name];

        handles.push(board::spawn(
            name.to_string(),
            board.board_ip.clone(),
            imposter_cfg.period_ms(name),
            imposter_cfg.udp_enabled(name),
            measurement_specs(board, imposter_cfg.random_step),
            data_packets(board).map(|p| (p.id, p.variables.clone())).collect(),
            dest,
            imposter_cfg.mode.clone(),
        ));
    }

    Ok(handles)
}

fn measurement_specs(board: &Board, random_step: f64) -> Vec<MeasurementSpec> {
    board
        .measurements
        .iter()
        .map(|m| MeasurementSpec {
            id: m.id.clone(),
            kind: m.kind.clone(),
            range: m.safe_range.or(m.warning_range),
            enum_count: m.enum_values.as_ref().map_or(0, |v| v.len()),
            random_step,
        })
        .collect()
}

fn data_packets(board: &Board) -> impl Iterator<Item = &Packet> {
    board
        .packets
        .iter()
        .filter(|p| matches!(p.kind, PacketType::Data))
}

fn backend_dest(config: &Config) -> anyhow::Result<SocketAddr> {
    let backend_ip = config
        .general_info
        .addresses
        .get("backend")
        .context("missing 'backend' in addresses")?;
    let udp_port = config
        .general_info
        .ports
        .get("UDP")
        .context("missing 'UDP' in ports")?;
    format!("{}:{}", backend_ip, udp_port)
        .parse()
        .context("invalid backend UDP address")
}
