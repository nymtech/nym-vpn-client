use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::debug;

/// RAII guard to ensure connection counter is decremented when dropped
pub struct ConnectionGuard {
    active_connections: Arc<RwLock<u32>>,
}

impl ConnectionGuard {
    pub async fn new(active_connections: Arc<RwLock<u32>>) -> Self {
        {
            let mut count = active_connections.write().await;
            *count += 1;
            debug!("Active connections: {}", *count);
        }
        Self { active_connections }
    }
}

impl Drop for ConnectionGuard {
    fn drop(&mut self) {
        let active_connections = self.active_connections.clone();
        tokio::spawn(async move {
            let mut count = active_connections.write().await;
            if *count > 0 {
                *count -= 1;
                debug!("Active connections: {}", *count);
            }
        });
    }
}
