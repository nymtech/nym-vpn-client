// Copyright 2021-2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: Apache-2.0

pub(crate) mod scalar_serde_helper {
    use bls12_381::Scalar;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use zeroize::Zeroizing;

    pub fn serialize<S: Serializer>(scalar: &Scalar, serializer: S) -> Result<S::Ok, S::Error> {
        scalar.to_bytes().serialize(serializer)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Scalar, D::Error> {
        let b = <[u8; 32]>::deserialize(deserializer)?;

        // make sure the bytes get zeroed
        let bytes = Zeroizing::new(b);

        let maybe_scalar: Option<Scalar> = Scalar::from_bytes(&bytes).into();
        maybe_scalar.ok_or(serde::de::Error::custom(
            "did not construct a valid bls12-381 scalar out of the provided bytes",
        ))
    }
}
