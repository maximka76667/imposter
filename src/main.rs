mod config;
mod imposter_cfg;

fn main() -> anyhow::Result<()> {
    let adj_dir = std::path::Path::new(
        r"C:\Users\MaxNB\AppData\Local\hyperloop-control-station\adj",
    );
    let cfg_path = std::path::Path::new(r"imposter.toml");

    let config = config::load(adj_dir)?;
    let imposter_cfg = imposter_cfg::load(cfg_path)?;

    for (name, board) in &config.boards {
        println!(
            "[ok] {} — ip: {}, {} measurements, {} packets, period: {}ms",
            name,
            board.board_ip,
            board.measurements.len(),
            board.packets.len(),
            imposter_cfg.period_ms(name),
        );
    }

    Ok(())
}
