// Copyright 2021 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: Apache-2.0

use bls12_381::hash_to_curve::{ExpandMsgXmd,
// HashToCurve,
HashToField};
use bls12_381::{G1Affine, G1Projective, G2Affine, G2Projective, Scalar};
// use ff::Field;

use crate::error::{CoconutError, Result};
// use crate::scheme::setup::Parameters;

// pub(crate) struct Polynomial {
//     coefficients: Vec<Scalar>,
// }
//
// impl Polynomial {
//     // for polynomial of degree n, we generate n+1 values
//     // (for example for degree 1, like y = x + 2, we need [2,1])
//     pub(crate) fn new_random(params: &Parameters, degree: u64) -> Self {
//         Polynomial {
//             coefficients: params.n_random_scalars((degree + 1) as usize),
//         }
//     }
//
//     /// Evaluates the polynomial at point x.
//     pub(crate) fn evaluate(&self, x: &Scalar) -> Scalar {
//         if self.coefficients.is_empty() {
//             Scalar::zero()
//             // if x is zero then we can ignore most of the expensive computation and
//             // just return the last term of the polynomial
//         } else if x.is_zero().into() {
//             // we checked that coefficients are not empty so unwrap here is fine
//             #[allow(clippy::unwrap_used)]
//             *self.coefficients.first().unwrap()
//         } else {
//             self.coefficients
//                 .iter()
//                 .enumerate()
//                 // coefficient[n] * x ^ n
//                 .map(|(i, coefficient)| coefficient * x.pow(&[i as u64, 0, 0, 0]))
//                 .sum()
//         }
//     }
// }

// A temporary way of hashing particular message into G1.
// Implementation idea was taken from `threshold_crypto`:
// https://github.com/poanetwork/threshold_crypto/blob/7709462f2df487ada3bb3243060504b5881f2628/src/lib.rs#L691
// Eventually it should get replaced by, most likely, the osswu map
// method once ideally it's implemented inside the pairing crate.

// note: I have absolutely no idea what are the correct domains for those. I just used whatever
// was given in the test vectors of `Hashing to Elliptic Curves draft-irtf-cfrg-hash-to-curve-11`

// https://datatracker.ietf.org/doc/html/draft-irtf-cfrg-hash-to-curve-11#appendix-J.9.1
// const G1_HASH_DOMAIN: &[u8] = b"QUUX-V01-CS02-with-BLS12381G1_XMD:SHA-256_SSWU_RO_";

// https://datatracker.ietf.org/doc/html/draft-irtf-cfrg-hash-to-curve-11#appendix-K.1
const SCALAR_HASH_DOMAIN: &[u8] = b"QUUX-V01-CS02-with-expander";

// pub(crate) fn hash_g1<M: AsRef<[u8]>>(msg: M) -> G1Projective {
//     <G1Projective as HashToCurve<ExpandMsgXmd<sha2::Sha256>>>::hash_to_curve(msg, G1_HASH_DOMAIN)
// }

pub fn hash_to_scalar<M: AsRef<[u8]>>(msg: M) -> Scalar {
    let mut output = vec![Scalar::zero()];

    Scalar::hash_to_field::<ExpandMsgXmd<sha2::Sha256>>(
        msg.as_ref(),
        SCALAR_HASH_DOMAIN,
        &mut output,
    );
    output[0]
}

pub(crate) fn try_deserialize_scalar_vec(
    expected_len: u64,
    bytes: &[u8],
    err: CoconutError,
) -> Result<Vec<Scalar>> {
    if bytes.len() != expected_len as usize * 32 {
        return Err(err);
    }

    let mut out = Vec::with_capacity(expected_len as usize);
    for i in 0..expected_len as usize {
        // we just checked we have exactly the amount of bytes we need and thus the unwrap is fine
        #[allow(clippy::unwrap_used)]
        let s_bytes = bytes[i * 32..(i + 1) * 32].try_into().unwrap();
        let s = match Into::<Option<Scalar>>::into(Scalar::from_bytes(&s_bytes)) {
            None => return Err(err),
            Some(scalar) => scalar,
        };
        out.push(s)
    }

    Ok(out)
}

pub(crate) fn try_deserialize_scalar(bytes: &[u8; 32], err: CoconutError) -> Result<Scalar> {
    Into::<Option<Scalar>>::into(Scalar::from_bytes(bytes)).ok_or(err)
}

pub(crate) fn try_deserialize_g1_projective(
    bytes: &[u8; 48],
    err: CoconutError,
) -> Result<G1Projective> {
    Into::<Option<G1Affine>>::into(G1Affine::from_compressed(bytes))
        .ok_or(err)
        .map(G1Projective::from)
}

pub(crate) fn try_deserialize_g2_projective(
    bytes: &[u8; 96],
    err: CoconutError,
) -> Result<G2Projective> {
    Into::<Option<G2Affine>>::into(G2Affine::from_compressed(bytes))
        .ok_or(err)
        .map(G2Projective::from)
}
