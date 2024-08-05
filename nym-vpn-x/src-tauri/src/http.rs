use std::time::Duration;

use once_cell::sync::Lazy;
use reqwest::Client;
use tracing::error;

pub static _HTTP_CLIENT: Lazy<Client> = Lazy::new(|| {
    Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
        .inspect_err(|e| {
            error!("Failed to create HTTP client: {:?}", e);
        })
        .unwrap()
});
