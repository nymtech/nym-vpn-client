// Copyright 2021 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: Apache-2.0

// use bls12_381::{
//     G1Projective, 
// };
// use group::{Curve, 
// };
//
// use crate::error::{CoconutError, Result};
// use crate::proofs::ProofCmCs;
//
// // TODO: possibly completely remove those two functions.
// // They only exist to have a simpler and smaller code snippets to test
// // basic functionalities.
// use crate::traits::{Base58, Bytable};
// use crate::utils::{
//     // hash_g1,
//     try_deserialize_g1_projective};

// TODO NAMING: double check this one
// Lambda
// #[derive(Debug)]
// #[cfg_attr(test, derive(PartialEq, Eq))]
// pub struct BlindSignRequest {
//     // cm
//     commitment: G1Projective,
//     // h
//     commitment_hash: G1Projective,
//     // c
//     private_attributes_commitments: Vec<G1Projective>,
//     // pi_s
//     pi_s: ProofCmCs,
// }

// impl TryFrom<&[u8]> for BlindSignRequest {
//     type Error = CoconutError;
//
//     fn try_from(bytes: &[u8]) -> Result<BlindSignRequest> {
//         if bytes.len() < 48 + 48 + 8 + 48 {
//             return Err(CoconutError::DeserializationMinLength {
//                 min: 48 + 48 + 8 + 48,
//                 actual: bytes.len(),
//             });
//         }
//
//         let mut j = 0;
//         let commitment_bytes_len = 48;
//         let commitment_hash_bytes_len = 48;
//
//         // safety: we made bound check and we're using constant offest
//         #[allow(clippy::unwrap_used)]
//         let cm_bytes = bytes[..j + commitment_bytes_len].try_into().unwrap();
//         let commitment = try_deserialize_g1_projective(
//             &cm_bytes,
//             CoconutError::Deserialization(
//                 "Failed to deserialize compressed commitment".to_string(),
//             ),
//         )?;
//         j += commitment_bytes_len;
//
//         // safety: we made bound check and we're using constant offest
//         #[allow(clippy::unwrap_used)]
//         let cm_hash_bytes = bytes[j..j + commitment_hash_bytes_len].try_into().unwrap();
//         let commitment_hash = try_deserialize_g1_projective(
//             &cm_hash_bytes,
//             CoconutError::Deserialization(
//                 "Failed to deserialize compressed commitment hash".to_string(),
//             ),
//         )?;
//         j += commitment_hash_bytes_len;
//
//         // safety: we made bound check and we're using constant offest
//         #[allow(clippy::unwrap_used)]
//         let c_len = u64::from_le_bytes(bytes[j..j + 8].try_into().unwrap());
//         j += 8;
//         if bytes[j..].len() < c_len as usize * 48 {
//             return Err(CoconutError::DeserializationMinLength {
//                 min: c_len as usize * 48,
//                 actual: bytes[56..].len(),
//             });
//         }
//
//         let mut private_attributes_commitments = Vec::with_capacity(c_len as usize);
//         for i in 0..c_len as usize {
//             let start = j + i * 48;
//             let end = start + 48;
//
//             if bytes.len() < end {
//                 return Err(CoconutError::Deserialization(
//                     "Failed to deserialize compressed commitment".to_string(),
//                 ));
//             }
//
//             // safety: we made bound check and we're using constant offest
//             #[allow(clippy::unwrap_used)]
//             let private_attributes_commitment_bytes = bytes[start..end].try_into().unwrap();
//             let private_attributes_commitment = try_deserialize_g1_projective(
//                 &private_attributes_commitment_bytes,
//                 CoconutError::Deserialization(
//                     "Failed to deserialize compressed commitment".to_string(),
//                 ),
//             )?;
//
//             private_attributes_commitments.push(private_attributes_commitment)
//         }
//
//         let pi_s = ProofCmCs::from_bytes(&bytes[j + c_len as usize * 48..])?;
//
//         Ok(BlindSignRequest {
//             commitment,
//             commitment_hash,
//             private_attributes_commitments,
//             pi_s,
//         })
//     }
// }
//
// impl Bytable for BlindSignRequest {
//     fn to_byte_vec(&self) -> Vec<u8> {
//         let cm_bytes = self.commitment.to_affine().to_compressed();
//         let cm_hash_bytes = self.commitment_hash.to_affine().to_compressed();
//         let c_len = self.private_attributes_commitments.len() as u64;
//         let proof_bytes = self.pi_s.to_bytes();
//
//         let mut bytes = Vec::with_capacity(48 + 48 + 8 + c_len as usize * 48 + proof_bytes.len());
//
//         bytes.extend_from_slice(&cm_bytes);
//         bytes.extend_from_slice(&cm_hash_bytes);
//         bytes.extend_from_slice(&c_len.to_le_bytes());
//         for c in &self.private_attributes_commitments {
//             bytes.extend_from_slice(&c.to_affine().to_compressed());
//         }
//
//         bytes.extend_from_slice(&proof_bytes);
//
//         bytes
//     }
//
//     fn try_from_byte_slice(slice: &[u8]) -> Result<Self> {
//         BlindSignRequest::from_bytes(slice)
//     }
// }
//
// impl Base58 for BlindSignRequest {}
//
// impl BlindSignRequest {
//     pub fn get_commitment_hash(&self) -> G1Projective {
//         self.commitment_hash
//     }
//
//     pub fn get_private_attributes_pedersen_commitments(&self) -> &[G1Projective] {
//         &self.private_attributes_commitments
//     }
//
//     pub fn to_bytes(&self) -> Vec<u8> {
//         self.to_byte_vec()
//     }
//
//     pub fn from_bytes(bytes: &[u8]) -> Result<BlindSignRequest> {
//         BlindSignRequest::try_from(bytes)
//     }
//
//     pub fn num_private_attributes(&self) -> usize {
//         self.private_attributes_commitments.len()
//     }
// }
