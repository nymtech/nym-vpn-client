// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

#[derive(thiserror::Error, Debug)]
pub enum AuthenticationError {
    #[cfg(target_os = "linux")]
    #[error("failed to get information about connected client")]
    GetSockOpt(#[source] nix::Error),

    #[cfg(target_os = "linux")]
    #[error("failed to create message bus connection")]
    MessageBusConnection(#[source] zbus::Error),

    #[cfg(target_os = "linux")]
    #[error("failed to create authority proxy")]
    AuthorityProxy(#[source] zbus::Error),

    #[cfg(target_os = "linux")]
    #[error("invalid number conversion")]
    NumberConversion(#[source] std::num::TryFromIntError),

    #[cfg(target_os = "linux")]
    #[error("failed to create subject")]
    Subject(#[source] zbus_polkit::Error),

    #[cfg(target_os = "linux")]
    #[error("failed to enumerate system polkit actions")]
    EnumerateActions(#[source] zbus::Error),

    #[cfg(target_os = "linux")]
    #[error("failed to create the polkit action policy file")]
    CreateActionPolicy(#[source] std::io::Error),

    #[cfg(target_os = "linux")]
    #[error("failed to write to disk the polkit action policy")]
    WriteActionPolicy(#[source] std::io::Error),

    #[cfg(target_os = "linux")]
    #[error("failed to check authorization")]
    CheckAuthorization(#[source] zbus::Error),

    #[cfg(target_os = "linux")]
    #[error("failed to cancel authorization")]
    CancelAuthorization(#[source] zbus::Error),

    #[cfg(target_os = "linux")]
    #[error("authorization timed out")]
    Timeout,

    #[cfg(target_os = "linux")]
    #[error("process is shutting down")]
    ShuttingDown,

    #[cfg(any(target_os = "linux", target_os = "windows"))]
    #[error("authorization denied")]
    AuthorizationDenied,
}
