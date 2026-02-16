use std::io::Result;

use tokio_stream::Stream;
use tokio_util::sync::CancellationToken;

use crate::{authentication, xpc::common::XpcConnection};

pub(crate) mod client;
pub(crate) mod common;
pub(crate) mod daemon;
pub(crate) mod local_spawner;

pub fn incoming(
    shutdown_token: CancellationToken,
) -> Result<impl Stream<Item = Result<XpcConnection>>> {
    let xpc_service = daemon::XpcService::spawn(shutdown_token)?;
    Ok(authentication::incoming(xpc_service))
}
