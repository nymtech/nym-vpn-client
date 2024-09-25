// Copyright 2023-2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::cmp::Ordering;

use nym_sdk::mixnet::ReconstructedMessage;

use crate::{error::Result, Error};

pub(crate) fn check_ipr_message_version(message: &ReconstructedMessage) -> Result<()> {
    // Switch this to to use the v7::VERSION constant as soon as it's available as pub
    // in the monorepo.
    let current_version = 7;

    // Assuing it's a IPR message, it will have a version as its first byte
    if let Some(version) = message.message.first() {
        match version.cmp(&current_version) {
            Ordering::Greater => Err(Error::ReceivedResponseWithNewVersion {
                expected: current_version,
                received: *version,
            }),
            Ordering::Less => Err(Error::ReceivedResponseWithOldVersion {
                expected: current_version,
                received: *version,
            }),
            Ordering::Equal => {
                // We're good
                Ok(())
            }
        }
    } else {
        Err(Error::NoVersionInMessage)
    }
}
