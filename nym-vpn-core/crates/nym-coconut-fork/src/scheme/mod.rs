// Copyright 2021 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: Apache-2.0

// TODO: implement https://crates.io/crates/signature traits?

use bls12_381::G1Projective;
use group::Curve;

pub(crate) use keygen::SecretKey;

use crate::error::{CoconutError, Result};
use crate::traits::{Base58, Bytable};
use crate::utils::try_deserialize_g1_projective;

pub(crate) mod keygen;

type SignerIndex = u64;

// (h, s)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Signature(G1Projective, G1Projective);

impl TryFrom<&[u8]> for Signature {
    type Error = CoconutError;

    fn try_from(bytes: &[u8]) -> Result<Signature> {
        if bytes.len() != 96 {
            return Err(CoconutError::Deserialization(format!(
                "Signature must be exactly 96 bytes, got {}",
                bytes.len()
            )));
        }

        // safety: we just checked for the length so the unwraps are fine
        #[allow(clippy::expect_used)]
        let sig1_bytes: &[u8; 48] = &bytes[..48].try_into().expect("Slice size != 48");
        #[allow(clippy::expect_used)]
        let sig2_bytes: &[u8; 48] = &bytes[48..].try_into().expect("Slice size != 48");

        let sig1 = try_deserialize_g1_projective(
            sig1_bytes,
            CoconutError::Deserialization("Failed to deserialize compressed sig1".to_string()),
        )?;

        let sig2 = try_deserialize_g1_projective(
            sig2_bytes,
            CoconutError::Deserialization("Failed to deserialize compressed sig2".to_string()),
        )?;

        Ok(Signature(sig1, sig2))
    }
}

impl Signature {
    fn to_bytes(self) -> [u8; 96] {
        let mut bytes = [0u8; 96];
        bytes[..48].copy_from_slice(&self.0.to_affine().to_compressed());
        bytes[48..].copy_from_slice(&self.1.to_affine().to_compressed());
        bytes
    }

    fn from_bytes(bytes: &[u8]) -> Result<Signature> {
        Signature::try_from(bytes)
    }
}

impl Bytable for Signature {
    fn to_byte_vec(&self) -> Vec<u8> {
        self.to_bytes().to_vec()
    }

    fn try_from_byte_slice(slice: &[u8]) -> Result<Self> {
        Signature::from_bytes(slice)
    }
}

impl Base58 for Signature {}
