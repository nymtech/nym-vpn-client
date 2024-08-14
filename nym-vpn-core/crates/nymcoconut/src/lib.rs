// Copyright 2021 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: Apache-2.0

#![warn(clippy::expect_used)]
#![warn(clippy::unwrap_used)]

// pub use bls12_381::Scalar;
// pub use elgamal::ElGamalKeyPair;
pub use elgamal::PublicKey;
pub use error::CoconutError;
pub use scheme::issuance::BlindSignRequest;
// pub use scheme::keygen::KeyPair;
pub use scheme::keygen::SecretKey;
pub use scheme::keygen::VerificationKey;
// pub use scheme::keygen::VerificationKeyShare;
// pub use scheme::setup::Parameters;
// pub use scheme::verification::BlindedSerialNumber;
pub use scheme::verification::VerifyCredentialRequest;
pub use scheme::BlindedSignature;
pub use scheme::Signature;
// pub use scheme::SignatureShare;
// pub use scheme::SignerIndex;
pub use traits::Base58;
pub use traits::Bytable;
pub use utils::hash_to_scalar;

pub(crate) mod elgamal;
mod error;
mod impls;
mod proofs;
mod scheme;
mod traits;
mod utils;

pub type Attribute = bls12_381::Scalar;
pub type PrivateAttribute = Attribute;
pub type PublicAttribute = Attribute;
//
// pub use bls12_381::G1Projective;
