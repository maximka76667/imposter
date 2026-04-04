mod config;
mod imposter_cfg;
mod udp;
mod watcher;

use anyhow::Context;

fn main() -> anyhow::Result<()> {
    println!();
    println!("╷╭┬╮╭─╮╭─╮╭─╮╶┬╴╭─╴╭─╮");
    println!("││││├─╯│ │╰─╮ │ ├╴ ├┬╯");
    println!("╵╵ ╵╵  ╰─╯╰─╯ ╵ ╰─╴╵╰╴");
    println!();
    tracing_subscriber::fmt::init();

    let base_dir = dirs::cache_dir()
        .expect("could not determine cache directory")
        .join("hyperloop-control-station");
    let adj_dir = base_dir.join("adj");
    let cfg_path = std::path::Path::new("imposter.toml");

    let branch = adj_branch(&adj_dir);
    tracing::info!(branch = %branch, "adj config");

    let config = config::load(&adj_dir)?;

    let imposter_cfg = imposter_cfg::load(cfg_path)?;
    launch_fleet(&config, &imposter_cfg)?;

    Ok(())
}

fn launch_fleet(
    config: &config::Config,
    imposter_cfg: &imposter_cfg::ImposterCfg,
) -> anyhow::Result<()> {
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

    for name in &names {
        let board = &config.boards[*name];
        tracing::info!(
            "> {} » {} - id: {} | measurements: {} | packets: {} | period: {}ms",
            board.board_ip,
            name,
            board.board_id,
            board.measurements.len(),
            board.packets.len(),
            imposter_cfg.period_ms(name),
        );
    }

    // Step 3: single board UDP smoke test — first board only
    let name = names[1];
    let board = &config.boards[name];
    let period = std::time::Duration::from_millis(imposter_cfg.period_ms(name));
    let socket = udp::bind(board)?;

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
    let dest: std::net::SocketAddr = format!("{}:{}", backend_ip, udp_port)
        .parse()
        .context("invalid backend UDP address")?;

    let measurements = udp::measurement_map(board);
    let data_packets: Vec<_> = udp::data_packets(board).collect();

    tracing::info!(ids = ?data_packets.iter().map(|p| p.id).collect::<Vec<_>>(), "data packets");
    tracing::info!(board = name, addr = %board.board_ip, packets = data_packets.len(), "starting UDP loop");

    loop {
        for packet in &data_packets {
            udp::send(&socket, packet, &measurements, dest)?;
        }
        std::thread::sleep(period);
    }
}

fn adj_branch(adj_dir: &std::path::Path) -> String {
    fn inner(adj_dir: &std::path::Path) -> Option<String> {
        let repo = git2::Repository::open(adj_dir).ok()?;
        let head = repo.head().ok()?;
        Some(head.shorthand().unwrap_or("unknown").to_string())
    }
    inner(adj_dir).unwrap_or_else(|| "unknown".to_string())
}
