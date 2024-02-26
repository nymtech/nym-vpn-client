use std::time::Duration;

use once_cell::sync::Lazy;
use reqwest::{Client as HttpClient, ClientBuilder, StatusCode};
use thiserror::Error;
use tracing::debug;

pub static HTTP_CLIENT: Lazy<HttpClient> = Lazy::new(|| {
    let timeout = Duration::new(5, 0);

    debug!("Creating HTTP client with default timeout: {:?}", timeout);
    ClientBuilder::new().timeout(timeout).build().unwrap()
});

#[derive(Error, Debug)]
pub enum HttpError {
    #[error("request failed [{0:?}]")]
    RequestError(Option<StatusCode>),
    #[error("failed to deserialize json response: {0}")]
    ResponseError(String),
}
