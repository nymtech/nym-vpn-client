use std::io;

use tokio::task::JoinSet;
use tokio_util::sync::CancellationToken;

pub fn install(shutdown_token: CancellationToken) -> JoinSet<()> {
    let mut join_set = JoinSet::new();

    let ctrlc_shutdown_token = shutdown_token.clone();
    join_set.spawn(async move {
        if let Err(e) = set_ctrlc_handler(ctrlc_shutdown_token).await {
            tracing::error!("Failed to set the ctrl-c handler: {}", e);
        }
    });

    #[cfg(unix)]
    join_set.spawn(async move {
        if let Err(e) = set_termination_handler(shutdown_token).await {
            tracing::error!("Failed to set the termination handler: {}", e);
        }
    });

    join_set
}

async fn set_ctrlc_handler(shutdown_token: CancellationToken) -> io::Result<()> {
    tokio::signal::ctrl_c().await?;
    tracing::info!("Received Ctrl-C signal.");
    shutdown_token.cancel();
    Ok(())
}

#[cfg(unix)]
async fn set_termination_handler(shutdown_token: CancellationToken) -> io::Result<()> {
    use tokio::signal::unix::{signal, SignalKind};

    let mut sigterm = signal(SignalKind::terminate())?;
    let mut sigquit = signal(SignalKind::quit())?;

    tokio::select! {
        _ = sigterm.recv() => {
            tracing::info!("Received SIGTERM signal.");
            shutdown_token.cancel();
        },
        _ = sigquit.recv() => {
            tracing::info!("Received SIGQUIT signal.");
            shutdown_token.cancel();
        }
    }

    shutdown_token.cancel();

    Ok(())
}
