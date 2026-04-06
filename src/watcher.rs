use std::path::PathBuf;
use std::time::Duration;

use anyhow::Result;
use notify::RecursiveMode;
use notify::Watcher;
use tokio::sync::mpsc;

use crate::imposter_cfg;
use crate::imposter_cfg::ImposterCfg;

pub async fn watch(
    cfg_path: PathBuf,
    on_reload: impl Fn(&ImposterCfg) + Send + 'static,
) -> Result<()> {
    let (tx, mut rx) = mpsc::channel::<notify::Result<notify::Event>>(16);

    let mut watcher = notify::recommended_watcher(move |res| {
        let _ = tx.blocking_send(res);
    })?;

    watcher.watch(&cfg_path, RecursiveMode::NonRecursive)?;

    tracing::info!(path = %cfg_path.display(), "watching imposter.toml");

    loop {
        match rx.recv().await {
            Some(_) => {
                loop {
                    match tokio::time::timeout(Duration::from_millis(300), rx.recv()).await {
                        Ok(Some(_)) => continue,
                        _ => break,
                    }
                }

                tracing::info!("imposter.toml changed, reloading");

                match imposter_cfg::load(&cfg_path) {
                    Ok(cfg) => on_reload(&cfg),
                    Err(e) => tracing::warn!(err = %e, "failed to reload imposter.toml"),
                }
                println!();
            }
            None => {
                tracing::error!("watcher channel closed");
                break;
            }
        }
    }

    Ok(())
}
