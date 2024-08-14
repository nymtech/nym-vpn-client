// Copyright 2021 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: Apache-2.0

// TODO: look at https://crates.io/crates/merlin to perhaps use it instead?

// use bls12_381::{
//     Scalar
// };
//
// use crate::error::{CoconutError, Result};
// use crate::utils::{try_deserialize_scalar, try_deserialize_scalar_vec};
//
// #[derive(Debug)]
// #[cfg_attr(test, derive(PartialEq, Eq))]
// pub struct ProofCmCs {
//     challenge: Scalar,
//     response_opening: Scalar,
//     response_openings: Vec<Scalar>,
//     response_attributes: Vec<Scalar>,
// }
//
// impl ProofCmCs {
//     // challenge || response opening || openings len || response openings || attributes len ||
//     // response attributes
//     pub(crate) fn to_bytes(&self) -> Vec<u8> {
//         let openings_len = self.response_openings.len() as u64;
//         let attributes_len = self.response_attributes.len() as u64;
//
//         let mut bytes = Vec::with_capacity(16 + (2 + openings_len + attributes_len) as usize * 32);
//
//         bytes.extend_from_slice(&self.challenge.to_bytes());
//         bytes.extend_from_slice(&self.response_opening.to_bytes());
//
//         bytes.extend_from_slice(&openings_len.to_le_bytes());
//         for ro in &self.response_openings {
//             bytes.extend_from_slice(&ro.to_bytes());
//         }
//
//         bytes.extend_from_slice(&attributes_len.to_le_bytes());
//         for rm in &self.response_attributes {
//             bytes.extend_from_slice(&rm.to_bytes());
//         }
//
//         bytes
//     }
//
//     pub(crate) fn from_bytes(bytes: &[u8]) -> Result<Self> {
//         // at the very minimum there must be a single attribute being proven
//         if bytes.len() < 32 * 4 + 16 || (bytes.len() - 16) % 32 != 0 {
//             return Err(CoconutError::Deserialization(
//                 "tried to deserialize proof of commitments with bytes of invalid length"
//                     .to_string(),
//             ));
//         }
//
//         let mut idx = 0;
//         // safety: bound checked + constant offset
//         #[allow(clippy::unwrap_used)]
//         let challenge_bytes = bytes[idx..idx + 32].try_into().unwrap();
//         idx += 32;
//         // safety: bound checked + constant offset
//         #[allow(clippy::unwrap_used)]
//         let response_opening_bytes = bytes[idx..idx + 32].try_into().unwrap();
//         idx += 32;
//
//         let challenge = try_deserialize_scalar(
//             &challenge_bytes,
//             CoconutError::Deserialization("Failed to deserialize challenge".to_string()),
//         )?;
//
//         let response_opening = try_deserialize_scalar(
//             &response_opening_bytes,
//             CoconutError::Deserialization(
//                 "Failed to deserialize the response to the random".to_string(),
//             ),
//         )?;
//
//         // safety: bound checked + constant offset
//         #[allow(clippy::unwrap_used)]
//         let ro_len = u64::from_le_bytes(bytes[idx..idx + 8].try_into().unwrap());
//         idx += 8;
//         if bytes[idx..].len() < ro_len as usize * 32 + 8 {
//             return Err(
//                 CoconutError::Deserialization(
//                     "tried to deserialize proof of ciphertexts and commitment with insufficient number of bytes provided".to_string()),
//             );
//         }
//
//         let ro_end = idx + ro_len as usize * 32;
//         let response_openings = try_deserialize_scalar_vec(
//             ro_len,
//             &bytes[idx..ro_end],
//             CoconutError::Deserialization("Failed to deserialize openings response".to_string()),
//         )?;
//
//         // safety: bound checked + constant offset
//         #[allow(clippy::unwrap_used)]
//         let rm_len = u64::from_le_bytes(bytes[ro_end..ro_end + 8].try_into().unwrap());
//         let response_attributes = try_deserialize_scalar_vec(
//             rm_len,
//             &bytes[ro_end + 8..],
//             CoconutError::Deserialization("Failed to deserialize attributes response".to_string()),
//         )?;
//
//         Ok(ProofCmCs {
//             challenge,
//             response_opening,
//             response_openings,
//             response_attributes,
//         })
//     }
// }
//
// #[derive(Debug, PartialEq, Eq)]
// pub struct ProofKappaZeta {
//     // c
//     challenge: Scalar,
//
//     // responses
//     response_serial_number: Scalar,
//     response_binding_number: Scalar,
//     response_blinder: Scalar,
// }
//
// impl ProofKappaZeta {
//     // challenge || response serial number || response binding number || repose blinder
//     pub(crate) fn to_bytes(&self) -> Vec<u8> {
//         let attributes_len = 2; // because we have serial number and the binding number
//         let mut bytes = Vec::with_capacity((1 + attributes_len + 1) as usize * 32);
//
//         bytes.extend_from_slice(&self.challenge.to_bytes());
//         bytes.extend_from_slice(&self.response_serial_number.to_bytes());
//         bytes.extend_from_slice(&self.response_binding_number.to_bytes());
//
//         bytes.extend_from_slice(&self.response_blinder.to_bytes());
//
//         bytes
//     }
//
//     pub(crate) fn from_bytes(bytes: &[u8]) -> Result<Self> {
//         // at the very minimum there must be a single attribute being proven
//         if bytes.len() != 128 {
//             return Err(CoconutError::DeserializationInvalidLength {
//                 actual: bytes.len(),
//                 modulus_target: bytes.len(),
//                 modulus: 32,
//                 object: "kappa and zeta".to_string(),
//                 target: 32 * 4,
//             });
//         }
//
//         // safety: bound checked + constant offset
//         #[allow(clippy::unwrap_used)]
//         let challenge_bytes = bytes[..32].try_into().unwrap();
//         let challenge = try_deserialize_scalar(
//             &challenge_bytes,
//             CoconutError::Deserialization("Failed to deserialize challenge".to_string()),
//         )?;
//
//         // safety: bound checked + constant offset
//         #[allow(clippy::unwrap_used)]
//         let serial_number_bytes = &bytes[32..64].try_into().unwrap();
//         let response_serial_number = try_deserialize_scalar(
//             serial_number_bytes,
//             CoconutError::Deserialization("failed to deserialize the serial number".to_string()),
//         )?;
//
//         // safety: bound checked + constant offset
//         #[allow(clippy::unwrap_used)]
//         let binding_number_bytes = &bytes[64..96].try_into().unwrap();
//         let response_binding_number = try_deserialize_scalar(
//             binding_number_bytes,
//             CoconutError::Deserialization("failed to deserialize the binding number".to_string()),
//         )?;
//
//         // safety: bound checked + constant offset
//         #[allow(clippy::unwrap_used)]
//         let blinder_bytes = bytes[96..].try_into().unwrap();
//         let response_blinder = try_deserialize_scalar(
//             &blinder_bytes,
//             CoconutError::Deserialization("failed to deserialize the blinder".to_string()),
//         )?;
//
//         Ok(ProofKappaZeta {
//             challenge,
//             response_serial_number,
//             response_binding_number,
//             response_blinder,
//         })
//     }
// }
//
// // proof builder:
// // - commitment
// // - challenge
// // - responses
//
