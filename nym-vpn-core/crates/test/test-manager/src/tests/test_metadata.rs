// Copyright 2016-2025 Mullvad VPN AB. All Rights Reserved.
// Copyright 2025 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use super::TestWrapperFunctionNym;
use test_rpc::meta::Os;

#[derive(Clone, Debug)]
pub struct TestMetadata {
    pub name: &'static str,
    pub targets: &'static [Os],
    pub func: TestWrapperFunctionNym,
    /// Priority order of the tests, unless specific tests are given as the `TEST_FILTERS` argument
    pub priority: Option<i32>,
    /// A list of location that will be used for by the test
    pub location: Option<Vec<String>>,
}

// Register our test metadata struct with inventory to allow submitting tests of this type.
inventory::collect!(TestMetadata);
