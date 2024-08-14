// Copyright 2021-2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: Apache-2.0

use crate::error::{CoconutError, Result};
use crate::proofs::ProofKappaZeta;
// use crate::scheme::setup::Parameters;
use crate::scheme::Signature;
// use crate::scheme::VerificationKey;
use crate::traits::{Base58, Bytable};
use crate::utils::try_deserialize_g2_projective;
// use crate::Attribute;
use bls12_381::{multi_miller_loop, G1Affine, G2Prepared, G2Projective, 
    // Scalar
};
use core::ops::Neg;
use group::{Curve, Group};

pub use crate::scheme::double_use::BlindedSerialNumber;

// TODO NAMING: this whole thing
// Theta
#[derive(Debug, PartialEq, Eq)]
pub struct VerifyCredentialRequest {
    // blinded_message (kappa)
    pub blinded_message: G2Projective,
    // blinded serial number (zeta)
    pub blinded_serial_number: BlindedSerialNumber,
    // sigma
    pub credential: Signature,
    // pi_v
    pub pi_v: ProofKappaZeta,
}

impl TryFrom<&[u8]> for VerifyCredentialRequest {
    type Error = CoconutError;

    fn try_from(bytes: &[u8]) -> Result<VerifyCredentialRequest> {
        if bytes.len() < 288 {
            return Err(
                CoconutError::Deserialization(
                    format!("Tried to deserialize theta with insufficient number of bytes, expected >= 288, got {}", bytes.len()),
                ));
        }

        // safety: we just checked for the length so the unwraps are fine
        #[allow(clippy::unwrap_used)]
        let blinded_message_bytes = bytes[..96].try_into().unwrap();
        let blinded_message = try_deserialize_g2_projective(
            &blinded_message_bytes,
            CoconutError::Deserialization(
                "failed to deserialize the blinded message (kappa)".to_string(),
            ),
        )?;

        let blinded_serial_number_bytes = &bytes[96..192];
        let blinded_serial_number =
            BlindedSerialNumber::try_from_byte_slice(blinded_serial_number_bytes)?;

        let credential = Signature::try_from(&bytes[192..288])?;

        let pi_v = ProofKappaZeta::from_bytes(&bytes[288..])?;

        Ok(VerifyCredentialRequest {
            blinded_message,
            blinded_serial_number,
            credential,
            pi_v,
        })
    }
}

impl VerifyCredentialRequest {
    // pub fn has_blinded_serial_number(&self, blinded_serial_number_bs58: &str) -> Result<bool> {
    //     let blinded_serial_number = BlindedSerialNumber::try_from_bs58(blinded_serial_number_bs58)?;
    //     let ret = self.blinded_serial_number.eq(&blinded_serial_number);
    //     Ok(ret)
    // }

    // blinded message (kappa)  || blinded serial number (zeta) || credential || pi_v
    pub fn to_bytes(&self) -> Vec<u8> {
        let blinded_message_bytes = self.blinded_message.to_affine().to_compressed();
        let blinded_serial_number_bytes = self.blinded_serial_number.to_affine().to_compressed();
        let credential_bytes = self.credential.to_bytes();
        let proof_bytes = self.pi_v.to_bytes();

        let mut bytes = Vec::with_capacity(288 + proof_bytes.len());
        bytes.extend_from_slice(&blinded_message_bytes);
        bytes.extend_from_slice(&blinded_serial_number_bytes);
        bytes.extend_from_slice(&credential_bytes);
        bytes.extend_from_slice(&proof_bytes);

        bytes
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<VerifyCredentialRequest> {
        VerifyCredentialRequest::try_from(bytes)
    }

    pub fn blinded_serial_number(&self) -> BlindedSerialNumber {
        self.blinded_serial_number
    }

    pub fn blinded_serial_number_bs58(&self) -> String {
        self.blinded_serial_number.to_bs58()
    }
}

impl Bytable for VerifyCredentialRequest {
    fn to_byte_vec(&self) -> Vec<u8> {
        self.to_bytes()
    }

    fn try_from_byte_slice(slice: &[u8]) -> Result<Self> {
        VerifyCredentialRequest::try_from(slice)
    }
}

impl Base58 for VerifyCredentialRequest {}

/// Checks whether e(P, Q) * e(-R, S) == id
pub fn check_bilinear_pairing(p: &G1Affine, q: &G2Prepared, r: &G1Affine, s: &G2Prepared) -> bool {
    // checking e(P, Q) * e(-R, S) == id
    // is equivalent to checking e(P, Q) == e(R, S)
    // but requires only a single final exponentiation rather than two of them
    // and therefore, as seen via benchmarks.rs, is almost 50% faster
    // (1.47ms vs 2.45ms, tested on R9 5900X)

    let multi_miller = multi_miller_loop(&[(p, q), (&r.neg(), s)]);
    multi_miller.final_exponentiation().is_identity().into()
}
