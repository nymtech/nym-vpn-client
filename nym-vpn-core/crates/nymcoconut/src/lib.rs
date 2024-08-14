// Copyright 2021 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: Apache-2.0

#![warn(clippy::expect_used)]
#![warn(clippy::unwrap_used)]

pub(crate) use error::CoconutError;
pub(crate) use scheme::keygen::VerificationKey;
pub use scheme::Signature;
pub(crate) use traits::Base58;
pub use utils::hash_to_scalar;

mod error;
mod impls;
mod scheme;
mod traits;
mod utils;

pub type Attribute = bls12_381::Scalar;
pub type PrivateAttribute = Attribute;
pub type PublicAttribute = Attribute;
