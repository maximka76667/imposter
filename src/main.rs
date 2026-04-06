mod board;
mod config;
mod fleet;
mod imposter_cfg;
mod simulator;
mod udp;
mod watcher;

use std::sync::Arc;

use tracing_subscriber::{filter::LevelFilter, prelude::*, reload};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!();
    println!("╷╭┬╮╭─╮╭─╮╭─╮╶┬╴╭─╴╭─╮");
    println!("││││├─╯│ │╰─╮ │ ├╴ ├┬╯");
    println!("╵╵ ╵╵  ╰─╯╰─╯ ╵ ╰─╴╵╰╴");
    println!();

    let (filter, filter_handle) = reload::Layer::new(LevelFilter::INFO);
    tracing_subscriber::registry()
        .with(filter)
        .with(tracing_subscriber::fmt::layer())
        .init();

    let base_dir = dirs::cache_dir()
        .expect("could not determine cache directory")
        .join("hyperloop-control-station");
    let adj_dir = base_dir.join("adj");
    let cfg_path = std::path::Path::new("imposter.toml");

    let branch = adj_branch(&adj_dir);
    tracing::info!(branch = %branch, "adj config");

    let config = config::load(&adj_dir)?;
    let imposter_cfg = imposter_cfg::load(cfg_path)?;

    if imposter_cfg.verbose {
        let _ = filter_handle.modify(|f| *f = LevelFilter::DEBUG);
    }

    let handles = Arc::new(fleet::launch(&config, &imposter_cfg)?);

    tokio::spawn(watcher::watch(cfg_path.to_path_buf(), move |cfg| {
        let level = if cfg.verbose { LevelFilter::DEBUG } else { LevelFilter::INFO };
        let _ = filter_handle.modify(|f| *f = level);

        for handle in handles.iter() {
            handle.set_period(cfg.period_ms(&handle.name));
            handle.set_udp(cfg.udp_enabled(&handle.name));
            handle.set_tcp(cfg.tcp_enabled(&handle.name));
        }

        tracing::info!("imposter.toml reloaded");
    }));

    tokio::signal::ctrl_c().await?;
    tracing::info!("shutting down");
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
