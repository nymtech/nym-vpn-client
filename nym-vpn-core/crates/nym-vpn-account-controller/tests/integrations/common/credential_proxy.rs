// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{
    collections::HashMap,
    str::FromStr,
    sync::{Arc, Mutex},
    time::Duration,
};

use nym_compact_ecash::scheme::{
    coin_indices_signatures::{aggregate_annotated_indices_signatures, sign_coin_indices},
    expiration_date_signatures::{aggregate_annotated_expiration_signatures, sign_expiration_date},
};
use nym_credential_proxy_requests::api::v1::ticketbook::models::{
    AggregatedCoinIndicesSignaturesResponse, AggregatedExpirationDateSignaturesResponse,
    MasterVerificationKeyResponse, PartialVerificationKey, PartialVerificationKeysResponse,
    TicketbookWalletSharesResponse, WalletShare,
};
use nym_credentials::{AggregatedCoinIndicesSignatures, AggregatedExpirationDateSignatures};
use nym_credentials_interface::{
    AnnotatedCoinIndexSignature, AnnotatedExpirationDateSignature, CoinIndexSignatureShare,
    ExpirationDateSignatureShare, TicketType,
};

use nym_compact_ecash::{
    Base58, EncodedDate, KeyPairAuth, PublicKeyUser, SecretKeyAuth, VerificationKeyAuth,
    WithdrawalRequest, aggregate_verification_keys, constants, issue, scheme::keygen::ttp_keygen,
    setup::Parameters,
};
use nym_ecash_time::EcashTime;
use nym_vpn_api_client::{
    request::RequestZkNymRequestBody,
    response::{NymVpnZkNym, NymVpnZkNymPost, NymVpnZkNymStatus, StatusOk},
};
use rand::distributions::{Alphanumeric, DistString};
use time::{Date, OffsetDateTime, macros::format_description};

use wiremock::{Request, Respond, ResponseTemplate};

/// This structure mocks the Credential proxy, because we have to actually signs the request, since they are inherently random
/// Cloning it keeps the same underlying available wallet shares so that multiple routes can act on it and properly simulate zk nym issuance
pub struct MockCredentialProxy {
    coin_indices_signatures: Vec<AnnotatedCoinIndexSignature>,
    expiration_date_signatures: Vec<AnnotatedExpirationDateSignature>,
    master_key: VerificationKeyAuth,
    authorities_keypairs: Vec<KeyPairAuth>,
    shares_available: Arc<Mutex<HashMap<String, NymVpnZkNym>>>,
}

impl Clone for MockCredentialProxy {
    fn clone(&self) -> Self {
        Self {
            coin_indices_signatures: self.coin_indices_signatures.clone(),
            expiration_date_signatures: self.expiration_date_signatures.clone(),
            master_key: self.master_key.clone(),
            authorities_keypairs: self
                .authorities_keypairs
                .iter()
                .map(|k| {
                    KeyPairAuth::new(
                        k.secret_key().clone(),
                        k.verification_key().clone(),
                        k.index,
                    )
                })
                .collect(),
            shares_available: self.shares_available.clone(),
        }
    }
}

impl MockCredentialProxy {
    pub fn new() -> anyhow::Result<MockCredentialProxy> {
        let total_coins = 10;
        let params = Parameters::new(total_coins);

        let expiration_date = nym_ecash_time::cred_exp_date().ecash_unix_timestamp();

        // generate authorities keys
        let authorities_keypairs = ttp_keygen(2, 3).unwrap();
        let indices: [u64; 3] = [1, 2, 3];
        let secret_keys_authorities: Vec<&SecretKeyAuth> = authorities_keypairs
            .iter()
            .map(|keypair| keypair.secret_key())
            .collect();
        let verification_keys_auth: Vec<VerificationKeyAuth> = authorities_keypairs
            .iter()
            .map(|keypair| keypair.verification_key())
            .collect();

        let verification_key =
            aggregate_verification_keys(&verification_keys_auth, Some(&[1, 2, 3]))?;

        // generate valid dates signatures
        let dates_signatures = generate_expiration_date_signatures(
            expiration_date,
            &secret_keys_authorities,
            &verification_keys_auth,
            &verification_key,
            &indices,
        )?;

        // generate coin indices signatures
        let coin_indices_signatures = generate_coin_indices_signatures(
            &params,
            &secret_keys_authorities,
            &verification_keys_auth,
            &verification_key,
            &indices,
        )?;

        Ok(MockCredentialProxy {
            coin_indices_signatures,
            expiration_date_signatures: dates_signatures,
            master_key: verification_key,
            authorities_keypairs,
            shares_available: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    fn issue_blinded_shares(
        &self,
        request: RequestZkNymRequestBody,
    ) -> anyhow::Result<Vec<WalletShare>> {
        let user_key = PublicKeyUser::from_base58_string(request.ecash_pubkey)?;
        let format = format_description!("[year]-[month]-[day]");
        let expiration_date =
            Date::parse(&request.expiration_date, &format)?.ecash_unix_timestamp();
        let t_type = TicketType::from_str(&request.ticketbook_type)?.encode();
        let req = WithdrawalRequest::try_from_bs58(request.withdrawal_request)?;
        let mut wallet_blinded_signatures = Vec::new();
        for auth_keypair in &self.authorities_keypairs {
            let blind_signature = issue(
                auth_keypair.secret_key(),
                user_key,
                &req,
                expiration_date,
                t_type,
            );
            wallet_blinded_signatures.push(WalletShare {
                node_index: auth_keypair.index.unwrap(),
                bs58_encoded_share: blind_signature.unwrap().to_bs58(),
            });
        }
        Ok(wallet_blinded_signatures)
    }

    fn coin_signatures(&self) -> AggregatedCoinIndicesSignaturesResponse {
        AggregatedCoinIndicesSignaturesResponse {
            signatures: AggregatedCoinIndicesSignatures {
                epoch_id: 0,
                signatures: self.coin_indices_signatures.clone(),
            },
        }
    }

    fn date_signatures(&self) -> AggregatedExpirationDateSignaturesResponse {
        AggregatedExpirationDateSignaturesResponse {
            signatures: AggregatedExpirationDateSignatures {
                epoch_id: 0,
                signatures: self.expiration_date_signatures.clone(),
                expiration_date: nym_ecash_time::cred_exp_date().ecash_date(),
            },
        }
    }

    fn master_key(&self) -> MasterVerificationKeyResponse {
        MasterVerificationKeyResponse {
            epoch_id: 0,
            bs58_encoded_key: self.master_key.to_bs58(),
        }
    }

    fn partial_verification_key(&self) -> PartialVerificationKeysResponse {
        PartialVerificationKeysResponse {
            epoch_id: 0,
            keys: self
                .authorities_keypairs
                .iter()
                .map(|k| PartialVerificationKey {
                    node_index: k.index.unwrap(),
                    bs58_encoded_key: k.verification_key().to_bs58(),
                })
                .collect(),
        }
    }

    pub fn zknym_id(self) -> impl Respond {
        move |req: &Request| {
            let zk_nym_id = req.url.path_segments().unwrap().next_back().unwrap();
            let guard = self.shares_available.lock().unwrap();
            if let Some(response) = guard.get(zk_nym_id) {
                ResponseTemplate::new(200).set_body_json(response)
            } else {
                ResponseTemplate::new(404)
            }
        }
    }

    pub fn zknym_post(self) -> impl Respond {
        move |req: &Request| {
            let request: RequestZkNymRequestBody = req.body_json().unwrap();
            let t_type = request.ticketbook_type.clone();
            let id = Alphanumeric.sample_string(&mut rand::thread_rng(), 15);

            // Prepare the actual response
            let wallet = TicketbookWalletSharesResponse {
                epoch_id: 0,
                shares: self.issue_blinded_shares(request).unwrap(),
                master_verification_key: Some(self.master_key()),
                aggregated_coin_index_signatures: Some(self.coin_signatures()),
                aggregated_expiration_date_signatures: Some(self.date_signatures()),
            };
            let now = OffsetDateTime::now_utc();
            let zk_nym = NymVpnZkNym {
                created_on_utc: now.to_string(),
                last_updated_utc: now.to_string(),
                id: id.clone(),
                ticketbook_type: t_type.clone(),
                valid_until_utc: (now + Duration::from_secs(3600 * 24 * 30)).to_string(),
                valid_from_utc: now.to_string(),
                issued_bandwidth_in_gb: 25f64,
                blinded_shares: Some(wallet),
                status: NymVpnZkNymStatus::Active,
            };
            let mut guard = self.shares_available.lock().unwrap();
            guard.insert(id.clone(), zk_nym);

            // respond as if it was pending
            let response = NymVpnZkNymPost {
                created_on_utc: now.to_string(),
                last_updated_utc: now.to_string(),
                id,
                ticketbook_type: t_type,
                valid_until_utc: (now + Duration::from_secs(3600 * 24 * 30)).to_string(),
                valid_from_utc: now.to_string(),
                issued_bandwidth_in_gb: 25f64,
                blinded_shares: None,
                status: NymVpnZkNymStatus::Pending,
            };
            ResponseTemplate::new(200).set_body_json(response)
        }
    }

    pub fn zknym_available_200(self) -> impl Respond {
        move |_req: &Request| {
            let guard = self.shares_available.lock().unwrap();
            let values = guard.values().collect::<Vec<_>>();
            ResponseTemplate::new(200).set_body_json(values)
        }
    }

    pub fn partial_verification_key_200(self) -> impl Respond {
        ResponseTemplate::new(200).set_body_json(self.partial_verification_key())
    }

    pub fn confirm_zk_nym_download_by_id_200(self) -> impl Respond {
        move |req: &Request| {
            let zk_nym_id = req.url.path_segments().unwrap().next_back().unwrap();
            let mut guard = self.shares_available.lock().unwrap();
            guard.remove(zk_nym_id);
            ResponseTemplate::new(200).set_body_json(StatusOk {
                status: "ok".to_string(),
            })
        }
    }
}

// These two helpers are copied from the ecash tests, because we need a slightly different return type
fn generate_expiration_date_signatures(
    expiration_date: EncodedDate,
    secret_keys_authorities: &[&SecretKeyAuth],
    verification_keys_auth: &[VerificationKeyAuth],
    verification_key: &VerificationKeyAuth,
    indices: &[u64],
) -> anyhow::Result<Vec<AnnotatedExpirationDateSignature>> {
    let mut edt_partial_signatures: Vec<Vec<_>> =
        Vec::with_capacity(constants::CRED_VALIDITY_PERIOD_DAYS as usize);
    for sk_auth in secret_keys_authorities.iter() {
        let sign = sign_expiration_date(sk_auth, expiration_date).unwrap();
        edt_partial_signatures.push(sign);
    }
    let combined_data: Vec<_> = indices
        .iter()
        .zip(
            verification_keys_auth
                .iter()
                .zip(edt_partial_signatures.iter()),
        )
        .map(|(i, (vk, sigs))| ExpirationDateSignatureShare {
            index: *i,
            key: vk.clone(),
            signatures: sigs.clone(),
        })
        .collect();

    Ok(aggregate_annotated_expiration_signatures(
        verification_key,
        expiration_date,
        &combined_data,
    )?)
}

fn generate_coin_indices_signatures(
    params: &Parameters,
    secret_keys_authorities: &[&SecretKeyAuth],
    verification_keys_auth: &[VerificationKeyAuth],
    verification_key: &VerificationKeyAuth,
    indices: &[u64],
) -> anyhow::Result<Vec<AnnotatedCoinIndexSignature>> {
    // create the partial signatures from each authority
    let partial_signatures: Vec<Vec<_>> = secret_keys_authorities
        .iter()
        .map(|sk_auth| sign_coin_indices(params, verification_key, sk_auth).unwrap())
        .collect();

    let combined_data: Vec<_> = indices
        .iter()
        .zip(verification_keys_auth.iter().zip(partial_signatures.iter()))
        .map(|(i, (vk, sigs))| CoinIndexSignatureShare {
            index: *i,
            key: vk.clone(),
            signatures: sigs.clone(),
        })
        .collect();

    Ok(aggregate_annotated_indices_signatures(
        params,
        verification_key,
        &combined_data,
    )?)
}
