use std::path::Path;
use std::sync::mpsc;
use std::time::Duration;

use anyhow::Result;
use notify::RecursiveMode;
use notify::Watcher;

use crate::imposter_cfg;
use crate::imposter_cfg::ImposterCfg;

pub fn watch(cfg_path: &Path, on_reload: impl Fn(&ImposterCfg)) -> Result<()> {
    let (tx, rx) = mpsc::channel::<notify::Result<notify::Event>>();

    let mut watcher = notify::recommended_watcher(move |res| {
        let _ = tx.send(res);
    })?;

    watcher.watch(cfg_path, RecursiveMode::NonRecursive)?;

    tracing::info!(path = %cfg_path.display(), "watching imposter.toml");

    loop {
        match rx.recv() {
            Ok(_) => {
                while rx.recv_timeout(Duration::from_millis(300)).is_ok() {}

                tracing::info!("imposter.toml changed, reloading");

                match imposter_cfg::load(cfg_path) {
                    Ok(cfg) => on_reload(&cfg),
                    Err(e) => tracing::warn!(err = %e, "failed to reload imposter.toml"),
                }
            }
            Err(e) => {
                tracing::error!(err = %e, "watcher channel closed");
                break;
            }
        }
    }

    Ok(())
}
