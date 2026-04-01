mod config;
mod imposter_cfg;

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let base_dir = dirs::cache_dir()
        .expect("could not determine cache directory")
        .join("hyperloop-control-station");
    let adj_dir = base_dir.join("adj");
    let cfg_path = std::path::Path::new("imposter.toml");

    let branch = adj_branch(&adj_dir);

    tracing::info!(branch = %branch, "adj config");

    let config = config::load(&adj_dir)?;
    let imposter_cfg = imposter_cfg::load(&cfg_path)?;

    for name in imposter_cfg.boards.keys() {
        if !config.boards.contains_key(name.as_str()) {
            tracing::warn!(board = %name, "unknown board in imposter.toml, ignoring");
        }
    }

    for (name, board) in &config.boards {
        tracing::info!(
            id = board.board_id,
            board = %name,
            ip = %board.board_ip,
            m = board.measurements.len(),
            p = board.packets.len(),
            period_ms = imposter_cfg.period_ms(name),
            "board loaded"
        );
    }

    Ok(())
}

fn adj_branch(adj_dir: &std::path::Path) -> String {
    fn inner(adj_dir: &std::path::Path) -> Option<String> {
        let repo = git2::Repository::open(adj_dir).ok()?;
        let head = repo.head().ok()?;
        Some(head.shorthand().unwrap_or("unknown").to_string())
    }
    inner(adj_dir).unwrap_or_else(|| "unknown".to_string())
}
