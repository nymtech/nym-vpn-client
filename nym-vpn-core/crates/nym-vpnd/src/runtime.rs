// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use tokio::runtime::{Builder, Runtime};

pub fn new_runtime() -> Runtime {
    Builder::new_multi_thread()
        .enable_all()
        .worker_threads(10)
        .build()
        .unwrap()
}
