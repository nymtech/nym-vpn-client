// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: Apache-2.0

use bls12_381::Scalar;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::str::FromStr;
use thiserror::Error;

pub use nym_coconut::{
    hash_to_scalar,
    Attribute, 
    Base58, 
    BlindSignRequest, 
    BlindedSerialNumber,
CoconutError,
Parameters,
PrivateAttribute,
PublicAttribute,
    Signature,
VerifyCredentialRequest,
};

pub const VOUCHER_INFO_TYPE: &str = "BandwidthVoucher";
pub const FREE_PASS_INFO_TYPE: &str = "FreeBandwidthPass";

#[derive(Debug, Error)]
#[error("{0} is not a valid credential type")]
pub struct UnknownCredentialType(String);

#[derive(Copy, Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum CredentialType {
    Voucher,
    FreePass,
}

impl FromStr for CredentialType {
    type Err = UnknownCredentialType;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == VOUCHER_INFO_TYPE {
            Ok(CredentialType::Voucher)
        } else if s == FREE_PASS_INFO_TYPE {
            Ok(CredentialType::FreePass)
        } else {
            Err(UnknownCredentialType(s.to_string()))
        }
    }
}

impl CredentialType {
    pub fn validate(&self, type_plain: &str) -> bool {
        match self {
            CredentialType::Voucher => type_plain == VOUCHER_INFO_TYPE,
            CredentialType::FreePass => type_plain == FREE_PASS_INFO_TYPE,
        }
    }

    pub fn is_free_pass(&self) -> bool {
        matches!(self, CredentialType::FreePass)
    }

    pub fn is_voucher(&self) -> bool {
        matches!(self, CredentialType::Voucher)
    }
}

impl Display for CredentialType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            CredentialType::Voucher => VOUCHER_INFO_TYPE.fmt(f),
            CredentialType::FreePass => FREE_PASS_INFO_TYPE.fmt(f),
        }
    }
}

#[derive(Debug, Clone)]
pub struct CredentialSigningData {
    pub pedersen_commitments_openings: Vec<Scalar>,

    pub blind_sign_request: BlindSignRequest,

    pub public_attributes_plain: Vec<String>,

    pub typ: CredentialType,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct CredentialSpendingData {
    pub embedded_private_attributes: usize,

    pub verify_credential_request: VerifyCredentialRequest,

    pub public_attributes_plain: Vec<String>,

    pub typ: CredentialType,

    /// The (DKG) epoch id under which the credential has been issued so that the verifier could use correct verification key for validation.
    pub epoch_id: u64,
}
