mod install;
mod service;

pub(crate) use service::start;
pub(crate) use service::{SERVICE_DESCRIPTION, SERVICE_DISPLAY_NAME, SERVICE_NAME};
