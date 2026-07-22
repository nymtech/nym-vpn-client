// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

//! VPN-API credential fetcher.
//!
//! Provides [`VpnApiCredentialFetcher`], a concrete
//! [`CredentialFetcher`](nym_bandwidth_controller::CredentialFetcher) implementation that acquires
//! zk-nym ticketbooks from the Nym VPN API. It is connectivity- and firewall-aware via an internal
//! [`state`] machine, and waits (rather than failing) until it is allowed to reach the API before
//! issuing requests.
//!
//! The issued ticketbooks are returned to the
//! [`BandwidthController`](nym_bandwidth_controller::BandwidthController), which owns the credential
//! storage; this crate owns only its [`storage`] of in-flight requests, used to resume them.

mod cached_data;
mod credential_request;
mod fetcher;
mod storage;
mod utils;

pub mod error;

pub use error::VpnApiFetcherError;
pub use fetcher::VpnApiCredentialFetcher;
