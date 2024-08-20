use tokio::sync::mpsc::UnboundedSender;

use super::{OSDefaultPath, OSDefaultPathObserver};

/// Observer type that funnels all new default routes into the channel.
#[derive(Debug)]
pub struct DefaultPathObserver {
    tx: UnboundedSender<OSDefaultPath>,
}

impl DefaultPathObserver {
    pub fn new(tx: UnboundedSender<OSDefaultPath>) -> Self {
        Self { tx }
    }
}

impl OSDefaultPathObserver for DefaultPathObserver {
    fn on_default_path_change(&self, new_path: OSDefaultPath) {
        if self.tx.send(new_path).is_err() {
            tracing::warn!("Failed to send default path change.");
        }
    }
}
