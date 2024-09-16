use talpid_core::dns::DnsMonitor;
use futures::channel::mpsc::UnboundedSender;

pub struct DnsHandler {
    inner: DnsMonitor,
}

impl DnsHandler {
    pub async fn new() {
        DnsMonitor::new(
            #[cfg(target_os = "linux")]
            tokio::runtime::Handle::current(),
            #[cfg(target_os = "linux")]

        )
    }
}
