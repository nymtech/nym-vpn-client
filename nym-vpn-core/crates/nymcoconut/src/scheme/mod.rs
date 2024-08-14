// Copyright 2021 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: Apache-2.0

// TODO: implement https://crates.io/crates/signature traits?

use bls12_381::{G1Projective, G2Prepared, G2Projective, Scalar};
use group::Curve;

pub(crate) use keygen::{SecretKey, VerificationKey};

use crate::error::{CoconutError, Result};
use crate::scheme::setup::Parameters;
use crate::scheme::verification::check_bilinear_pairing;
use crate::traits::{Base58, Bytable};
use crate::utils::try_deserialize_g1_projective;
use crate::Attribute;

// mod aggregation;
mod double_use;
// pub(crate) mod issuance;
pub(crate) mod keygen;
pub(crate) mod setup;
mod verification;

pub(crate) type SignerIndex = u64;

// (h, s)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Signature(pub(crate) G1Projective, pub(crate) G1Projective);

// pub type PartialSignature = Signature;

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
    // pub(crate) fn sig1(&self) -> &G1Projective {
    //     &self.0
    // }
    //
    // pub(crate) fn sig2(&self) -> &G1Projective {
    //     &self.1
    // }

    pub fn randomise_simple(&self, params: &Parameters) -> Signature {
        let r = params.random_scalar();
        Signature(self.0 * r, self.1 * r)
    }

    pub fn randomise(&self, params: &Parameters) -> (Signature, Scalar) {
        let r = params.random_scalar();
        let r_prime = params.random_scalar();
        let h_prime = self.0 * r_prime;
        let s_prime = (self.1 * r_prime) + (h_prime * r);
        (Signature(h_prime, s_prime), r)
    }

    pub fn to_bytes(self) -> [u8; 96] {
        let mut bytes = [0u8; 96];
        bytes[..48].copy_from_slice(&self.0.to_affine().to_compressed());
        bytes[48..].copy_from_slice(&self.1.to_affine().to_compressed());
        bytes
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Signature> {
        Signature::try_from(bytes)
    }

    pub fn verify(
        &self,
        params: &Parameters,
        partial_verification_key: &VerificationKey,
        private_attributes: &[&Attribute],
        public_attributes: &[&Attribute],
        commitment_hash: &G1Projective,
    ) -> Result<()> {
        // Verify the commitment hash
        if !(commitment_hash == &self.0) {
            return Err(CoconutError::Verification(
                "Verification of commitment hash from signature failed".to_string(),
            ));
        }

        let alpha = partial_verification_key.alpha;

        let signed_attributes = private_attributes
            .iter()
            .chain(public_attributes.iter())
            .zip(partial_verification_key.beta_g2.iter())
            .map(|(&attr, beta_i)| beta_i * attr)
            .sum::<G2Projective>();

        // Verify the signature share
        if !check_bilinear_pairing(
            &self.0.to_affine(),
            &G2Prepared::from((alpha + signed_attributes).to_affine()),
            &self.1.to_affine(),
            params.prepared_miller_g2(),
        ) {
            return Err(CoconutError::Unblind(
                "Verification of signature share failed".to_string(),
            ));
        }

        Ok(())
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

#[derive(Debug, PartialEq, Eq)]
pub struct BlindedSignature(G1Projective, G1Projective);

impl Bytable for BlindedSignature {
    fn to_byte_vec(&self) -> Vec<u8> {
        self.to_bytes().to_vec()
    }

    fn try_from_byte_slice(slice: &[u8]) -> Result<Self> {
        Self::from_bytes(slice)
    }
}

impl Base58 for BlindedSignature {}

impl TryFrom<&[u8]> for BlindedSignature {
    type Error = CoconutError;

    fn try_from(bytes: &[u8]) -> Result<BlindedSignature> {
        if bytes.len() != 96 {
            return Err(CoconutError::Deserialization(format!(
                "BlindedSignature must be exactly 96 bytes, got {}",
                bytes.len()
            )));
        }

        // safety: we just checked for the length so the unwraps are fine
        #[allow(clippy::expect_used)]
        let h_bytes: &[u8; 48] = &bytes[..48].try_into().expect("Slice size != 48");
        #[allow(clippy::expect_used)]
        let sig_bytes: &[u8; 48] = &bytes[48..].try_into().expect("Slice size != 48");

        let h = try_deserialize_g1_projective(
            h_bytes,
            CoconutError::Deserialization("Failed to deserialize compressed h".to_string()),
        )?;
        let sig = try_deserialize_g1_projective(
            sig_bytes,
            CoconutError::Deserialization("Failed to deserialize compressed sig".to_string()),
        )?;

        Ok(BlindedSignature(h, sig))
    }
}

impl BlindedSignature {
    pub fn unblind(
        &self,
        partial_verification_key: &VerificationKey,
        pedersen_commitments_openings: &[Scalar],
    ) -> Signature {
        // parse the signature
        let h = &self.0;
        let c = &self.1;
        let blinding_removers = partial_verification_key
            .beta_g1
            .iter()
            .zip(pedersen_commitments_openings.iter())
            .map(|(beta, opening)| beta * opening)
            .sum::<G1Projective>();

        let unblinded_c = c - blinding_removers;

        Signature(*h, unblinded_c)
    }

    pub fn unblind_and_verify(
        &self,
        params: &Parameters,
        partial_verification_key: &VerificationKey,
        private_attributes: &[&Attribute],
        public_attributes: &[&Attribute],
        commitment_hash: &G1Projective,
        pedersen_commitments_openings: &[Scalar],
    ) -> Result<Signature> {
        let unblinded = self.unblind(partial_verification_key, pedersen_commitments_openings);
        unblinded.verify(
            params,
            partial_verification_key,
            private_attributes,
            public_attributes,
            commitment_hash,
        )?;
        Ok(unblinded)
    }

    pub fn to_bytes(&self) -> [u8; 96] {
        let mut bytes = [0u8; 96];
        bytes[..48].copy_from_slice(&self.0.to_affine().to_compressed());
        bytes[48..].copy_from_slice(&self.1.to_affine().to_compressed());
        bytes
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<BlindedSignature> {
        BlindedSignature::try_from(bytes)
    }
}

// perhaps this should take signature by reference? we'll see how it goes
#[derive(Clone, Copy)]
pub struct SignatureShare {
    signature: Signature,
    index: SignerIndex,
}

impl From<(Signature, SignerIndex)> for SignatureShare {
    fn from(value: (Signature, SignerIndex)) -> Self {
        SignatureShare {
            signature: value.0,
            index: value.1,
        }
    }
}

impl SignatureShare {
    pub fn new(signature: Signature, index: SignerIndex) -> Self {
        SignatureShare { signature, index }
    }

    pub fn signature(&self) -> &Signature {
        &self.signature
    }

    pub fn index(&self) -> SignerIndex {
        self.index
    }
}

