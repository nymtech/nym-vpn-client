// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: Apache-2.0

use crate::{NodeIdentity, Recipient, UniffiCustomTypeConverter};

uniffi::custom_type!(Recipient, String);

impl UniffiCustomTypeConverter for Recipient {
    type Builtin = String;

    fn into_custom(val: Self::Builtin) -> uniffi::Result<Self> {
        Ok(Recipient::try_from_base58_string(val)?)
    }

    fn from_custom(obj: Self) -> Self::Builtin {
        obj.to_string()
    }
}
