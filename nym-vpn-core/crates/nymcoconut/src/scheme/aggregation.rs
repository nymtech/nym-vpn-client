// Copyright 2021 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: Apache-2.0

use core::iter::Sum;
use core::ops::Mul;

use bls12_381::{
    // G2Prepared, G2Projective,
    Scalar
};
// use group::Curve;
use itertools::Itertools;

use crate::error::{CoconutError, Result};
// use crate::scheme::verification::check_bilinear_pairing;
use crate::scheme::{PartialSignature, Signature,
// SignatureShare,
SignerIndex, VerificationKey};
use crate::utils::perform_lagrangian_interpolation_at_origin;
// use crate::{Attribute, Parameters, VerificationKeyShare};

pub(crate) trait Aggregatable: Sized {
    fn aggregate(aggregatable: &[Self], indices: Option<&[SignerIndex]>) -> Result<Self>;

    fn check_unique_indices(indices: &[SignerIndex]) -> bool {
        // if aggregation is a threshold one, all indices should be unique
        indices.iter().unique_by(|&index| index).count() == indices.len()
    }
}

// includes `VerificationKey`
impl<T> Aggregatable for T
where
    T: Sum,
    for<'a> T: Sum<&'a T>,
    for<'a> &'a T: Mul<Scalar, Output = T>,
{
    fn aggregate(aggregatable: &[T], indices: Option<&[u64]>) -> Result<T> {
        if aggregatable.is_empty() {
            return Err(CoconutError::Aggregation("Empty set of values".to_string()));
        }

        if let Some(indices) = indices {
            if !Self::check_unique_indices(indices) {
                return Err(CoconutError::Aggregation("Non-unique indices".to_string()));
            }
            perform_lagrangian_interpolation_at_origin(indices, aggregatable)
        } else {
            // non-threshold
            Ok(aggregatable.iter().sum())
        }
    }
}

impl Aggregatable for PartialSignature {
    fn aggregate(sigs: &[PartialSignature], indices: Option<&[u64]>) -> Result<Signature> {
        let h = sigs
            .first()
            .ok_or_else(|| CoconutError::Aggregation("Empty set of signatures".to_string()))?
            .sig1();

        // TODO: is it possible to avoid this allocation?
        let sigmas = sigs.iter().map(|sig| *sig.sig2()).collect::<Vec<_>>();
        let aggr_sigma = Aggregatable::aggregate(&sigmas, indices)?;

        Ok(Signature(*h, aggr_sigma))
    }
}

/// Ensures all provided verification keys were generated to verify the same number of attributes.
fn check_same_key_size(keys: &[VerificationKey]) -> bool {
    keys.iter().map(|vk| vk.beta_g1.len()).all_equal()
        && keys.iter().map(|vk| vk.beta_g2.len()).all_equal()
}

pub fn aggregate_verification_keys(
    keys: &[VerificationKey],
    indices: Option<&[SignerIndex]>,
) -> Result<VerificationKey> {
    if !check_same_key_size(keys) {
        return Err(CoconutError::Aggregation(
            "Verification keys are of different sizes".to_string(),
        ));
    }
    Aggregatable::aggregate(keys, indices)
}

// pub fn aggregate_key_shares(shares: &[VerificationKeyShare]) -> Result<VerificationKey> {
//     let (keys, indices): (Vec<_>, Vec<_>) = shares
//         .iter()
//         .map(|share| (share.key.clone(), share.index))
//         .unzip();
//
//     aggregate_verification_keys(&keys, Some(&indices))
// }
//
// pub fn aggregate_signatures(
//     signatures: &[PartialSignature],
//     indices: Option<&[SignerIndex]>,
// ) -> Result<Signature> {
//     Aggregatable::aggregate(signatures, indices)
// }
//
// pub fn aggregate_signatures_and_verify(
//     params: &Parameters,
//     verification_key: &VerificationKey,
//     attributes: &[&Attribute],
//     signatures: &[PartialSignature],
//     indices: Option<&[SignerIndex]>,
// ) -> Result<Signature> {
//     // aggregate the signature
//     let signature = aggregate_signatures(signatures, indices)?;
//
//     // Verify the signature
//     let alpha = verification_key.alpha;
//
//     let tmp = attributes
//         .iter()
//         .zip(verification_key.beta_g2.iter())
//         .map(|(&attr, beta_i)| beta_i * attr)
//         .sum::<G2Projective>();
//
//     if !check_bilinear_pairing(
//         &signature.0.to_affine(),
//         &G2Prepared::from((alpha + tmp).to_affine()),
//         &signature.1.to_affine(),
//         params.prepared_miller_g2(),
//     ) {
//         return Err(CoconutError::Aggregation(
//             "Verification of the aggregated signature failed".to_string(),
//         ));
//     }
//     Ok(signature)
// }
//
// pub fn aggregate_signature_shares(shares: &[SignatureShare]) -> Result<Signature> {
//     let (signatures, indices): (Vec<_>, Vec<_>) = shares
//         .iter()
//         .map(|share| (*share.signature(), share.index()))
//         .unzip();
//
//     aggregate_signatures(&signatures, Some(&indices))
// }
//
// pub fn aggregate_signature_shares_and_verify(
//     params: &Parameters,
//     verification_key: &VerificationKey,
//     attributes: &[&Attribute],
//     shares: &[SignatureShare],
// ) -> Result<Signature> {
//     let (signatures, indices): (Vec<_>, Vec<_>) = shares
//         .iter()
//         .map(|share| (*share.signature(), share.index()))
//         .unzip();
//
//     aggregate_signatures_and_verify(
//         params,
//         verification_key,
//         attributes,
//         &signatures,
//         Some(&indices),
//     )
// }

