// Copyright 2021 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: Apache-2.0

use bls12_381::hash_to_curve::{ExpandMsgXmd, HashToField};
use bls12_381::{G1Affine, G1Projective, G2Affine, G2Projective, Scalar};

use crate::error::{CoconutError, Result};

// https://datatracker.ietf.org/doc/html/draft-irtf-cfrg-hash-to-curve-11#appendix-K.1
const SCALAR_HASH_DOMAIN: &[u8] = b"QUUX-V01-CS02-with-expander";

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
