// Copyright 2016-2025 Mullvad VPN AB. All Rights Reserved.
// Copyright 2025 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

/// Creates a new result type that returns the given result variant on error.
macro_rules! ffi_error {
    ($result:ident, $error:expr) => {
        #[repr(C)]
        #[derive(Debug)]
        pub struct $result {
            success: bool,
        }

        impl $result {
            pub fn into_result(self) -> Result<(), Error> {
                match self.success {
                    true => Ok(()),
                    false => Err($error),
                }
            }
        }

        impl From<$result> for Result<(), Error> {
            fn from(result: $result) -> Self {
                result.into_result()
            }
        }
    };
}
