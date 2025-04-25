// Copyright 2016-2024 Mullvad VPN AB. All Rights Reserved.
// Copyright 2024 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{error::Error, fmt, fmt::Write};

/// Used to generate string representations of error chains.
pub trait ErrorExt {
    /// Creates a string representation of the entire error chain.
    fn display_chain(&self) -> String;

    /// Like [Self::display_chain] but with an extra message at the start of the chain.
    fn display_chain_with_msg<S: AsRef<str>>(&self, msg: S) -> String;

    /// Print error chain to log using error level.
    fn trace_chain(&self);

    /// Like [Self::trace_chain] but with an extra message at the start of the chain.
    fn trace_chain_with_msg<S: AsRef<str>>(&self, msg: S);
}

impl<E: Error> ErrorExt for E {
    fn display_chain(&self) -> String {
        let mut s = format!("Error: {self}");
        let mut source = self.source();
        while let Some(error) = source {
            write!(&mut s, "\nCaused by: {error}").expect("formatting failed");
            source = error.source();
        }
        s
    }

    fn display_chain_with_msg<S: AsRef<str>>(&self, msg: S) -> String {
        let mut s = format!("Error: {}\nCaused by: {}", msg.as_ref(), self);
        let mut source = self.source();
        while let Some(error) = source {
            write!(&mut s, "\nCaused by: {error}").expect("formatting failed");
            source = error.source();
        }
        s
    }

    fn trace_chain(&self) {
        tracing::error!("{}", self.display_chain());
    }

    fn trace_chain_with_msg<S: AsRef<str>>(&self, msg: S) {
        tracing::error!("{}", self.display_chain_with_msg(msg));
    }
}

#[derive(Debug)]
pub struct BoxedError(Box<dyn Error + 'static + Send>);

impl fmt::Display for BoxedError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl Error for BoxedError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.0.source()
    }
}

impl BoxedError {
    pub fn new(error: impl Error + 'static + Send) -> Self {
        BoxedError(Box::new(error))
    }
}
