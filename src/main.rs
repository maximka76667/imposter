mod config;
mod imposter_cfg;

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let adj_dir = std::path::Path::new(
        r"C:\Users\MaxNB\AppData\Local\hyperloop-control-station\adj",
    );
    let cfg_path = std::path::Path::new(r"imposter.toml");

    let config = config::load(adj_dir)?;
    let imposter_cfg = imposter_cfg::load(cfg_path)?;

    for name in imposter_cfg.boards.keys() {
        if !config.boards.contains_key(name.as_str()) {
            tracing::warn!(board = %name, "unknown board in imposter.toml, ignoring");
        }
    }

    for (name, board) in &config.boards {
        tracing::info!(
            board = %name,
            ip = %board.board_ip,
            measurements = board.measurements.len(),
            packets = board.packets.len(),
            period_ms = imposter_cfg.period_ms(name),
            "board loaded"
        );
    }

    Ok(())
}
