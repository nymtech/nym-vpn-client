// Copyright 2021 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: Apache-2.0

#![warn(clippy::expect_used)]
#![warn(clippy::unwrap_used)]

pub use error::CoconutError;
pub use scheme::keygen::VerificationKey;
pub use scheme::BlindedSignature;
pub use scheme::Signature;
pub use traits::Base58;
pub use traits::Bytable;
pub use utils::hash_to_scalar;

mod error;
mod impls;
mod proofs;
mod scheme;
mod traits;
mod utils;

pub type Attribute = bls12_381::Scalar;
pub type PrivateAttribute = Attribute;
pub type PublicAttribute = Attribute;
