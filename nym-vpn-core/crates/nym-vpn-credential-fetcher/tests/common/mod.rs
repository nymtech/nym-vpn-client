// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

//! A mock VPN API for exercising the credential fetcher end to end: it issues
//! cryptographically valid zk-nym ticketbooks (the ecash fixture is adapted from the
//! nym-vpn-account-controller integration tests' MockCredentialProxy) while letting each test
//! control when a requested zk-nym transitions from `Pending` to `Active` or `Error`.

use std::{
    collections::HashMap,
    str::FromStr,
    sync::{Arc, Mutex},
    time::Duration,
};

use nym_compact_ecash::{
    Base58, EncodedDate, KeyPairAuth, PublicKeyUser, SecretKeyAuth, VerificationKeyAuth,
    WithdrawalRequest, aggregate_verification_keys, constants, issue,
    scheme::{
        coin_indices_signatures::{aggregate_annotated_indices_signatures, sign_coin_indices},
        expiration_date_signatures::{
            aggregate_annotated_expiration_signatures, sign_expiration_date,
        },
        keygen::ttp_keygen,
    },
    setup::Parameters,
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
use nym_ecash_time::EcashTime;
use nym_vpn_api_client::{
    request::RequestZkNymRequestBody,
    response::{
        NymVpnHealthResponse, NymVpnZkNym, NymVpnZkNymPost, NymVpnZkNymResponse, NymVpnZkNymStatus,
        StatusOk,
    },
};
use rand::distributions::{Alphanumeric, DistString};
use time::{Date, OffsetDateTime, macros::format_description};
use wiremock::{
    Mock, MockServer, Request, Respond, ResponseTemplate,
    matchers::{method, path, path_regex},
};

const ACCOUNT_REGEX: &str = r"n1\w*";
const DEVICE_REGEX: &str = r"\w*";
// Issued ids are 15 alphanumeric characters, so this can never match the `available` sub-route.
const ZK_NYM_ID_REGEX: &str = r"[A-Za-z0-9]{15}";

struct ZkNymRecord {
    status: NymVpnZkNymStatus,
    request: RequestZkNymRequestBody,
}

#[derive(Default)]
struct State {
    records: HashMap<String, ZkNymRecord>,
    /// When set, the next poll of any zk-nym id first flips every `Pending` record to `Active`.
    activate_on_next_poll: bool,
    /// Every polled id, in order of arrival.
    polls: Vec<String>,
    posts: usize,
}

/// The crypto material and mutable zk-nym registry backing the mock VPN API.
pub struct MockZkNymApi {
    coin_indices_signatures: Vec<AnnotatedCoinIndexSignature>,
    expiration_date_signatures: Vec<AnnotatedExpirationDateSignature>,
    master_key: VerificationKeyAuth,
    authorities_keypairs: Vec<KeyPairAuth>,
    state: Arc<Mutex<State>>,
}

impl Clone for MockZkNymApi {
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
            state: self.state.clone(),
        }
    }
}

impl MockZkNymApi {
    pub fn new() -> anyhow::Result<MockZkNymApi> {
        let total_coins = 10;
        let params = Parameters::new(total_coins);

        let expiration_date = nym_ecash_time::cred_exp_date().ecash_unix_timestamp();

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

        let dates_signatures = generate_expiration_date_signatures(
            expiration_date,
            &secret_keys_authorities,
            &verification_keys_auth,
            &verification_key,
            &indices,
        )?;

        let coin_indices_signatures = generate_coin_indices_signatures(
            &params,
            &secret_keys_authorities,
            &verification_keys_auth,
            &verification_key,
            &indices,
        )?;

        Ok(MockZkNymApi {
            coin_indices_signatures,
            expiration_date_signatures: dates_signatures,
            master_key: verification_key,
            authorities_keypairs,
            state: Arc::new(Mutex::new(State::default())),
        })
    }

    /// Register every route the credential fetcher needs on `server`.
    pub async fn register(&self, server: &MockServer) {
        let zknym_path = format!("^/public/v1/account/{ACCOUNT_REGEX}/device/{DEVICE_REGEX}/zknym");

        server
            .register(
                Mock::given(method("GET"))
                    .and(path("/public/v1/health"))
                    .respond_with(
                        ResponseTemplate::new(200).set_body_json(NymVpnHealthResponse {
                            status: "ok".to_string(),
                            timestamp_utc: OffsetDateTime::now_utc(),
                        }),
                    ),
            )
            .await;
        server
            .register(
                Mock::given(method("POST"))
                    .and(path_regex(format!("{zknym_path}$")))
                    .respond_with(self.clone().post_zknym()),
            )
            .await;
        server
            .register(
                Mock::given(method("GET"))
                    .and(path_regex(format!("{zknym_path}$")))
                    .respond_with(self.clone().list_zknyms()),
            )
            .await;
        server
            .register(
                Mock::given(method("GET"))
                    .and(path_regex(format!("{zknym_path}/available$")))
                    .respond_with(self.clone().list_available_zknyms()),
            )
            .await;
        server
            .register(
                Mock::given(method("GET"))
                    .and(path_regex(format!("{zknym_path}/{ZK_NYM_ID_REGEX}$")))
                    .respond_with(self.clone().poll_zknym()),
            )
            .await;
        server
            .register(
                Mock::given(method("DELETE"))
                    .and(path_regex(format!("{zknym_path}/{ZK_NYM_ID_REGEX}$")))
                    .respond_with(ResponseTemplate::new(200).set_body_json(StatusOk {
                        status: "ok".to_string(),
                    })),
            )
            .await;
        server
            .register(
                Mock::given(method("GET"))
                    .and(path(
                        "/public/v1/directory/zk-nyms/ticketbook/partial-verification-keys",
                    ))
                    .respond_with(
                        ResponseTemplate::new(200).set_body_json(self.partial_verification_keys()),
                    ),
            )
            .await;
        server
            .register(
                Mock::given(method("GET"))
                    .and(path(
                        "/public/v1/directory/zk-nyms/ticketbook/master-verification-key",
                    ))
                    .respond_with(ResponseTemplate::new(200).set_body_json(
                        MasterVerificationKeyResponse {
                            epoch_id: 0,
                            bs58_encoded_key: self.master_key.to_bs58(),
                        },
                    )),
            )
            .await;
    }

    //---------------------
    // Test controls
    //---------------------

    /// Make the next poll of any zk-nym id flip every `Pending` record to `Active` first, so the
    /// poll observes a finished issuance.
    pub fn activate_pending_on_next_poll(&self) {
        self.state.lock().unwrap().activate_on_next_poll = true;
    }

    /// Mark every record as failed server-side: subsequent polls return status `Error`.
    pub fn fail_all_requests(&self) {
        for record in self.state.lock().unwrap().records.values_mut() {
            record.status = NymVpnZkNymStatus::Error;
        }
    }

    /// Number of zk-nym creation requests (POSTs) received so far.
    pub fn post_count(&self) -> usize {
        self.state.lock().unwrap().posts
    }

    /// Wait until at least `count` polls have been received, panicking after `timeout`.
    pub async fn wait_for_polls(&self, count: usize, timeout: Duration) {
        let deadline = tokio::time::Instant::now() + timeout;
        loop {
            if self.state.lock().unwrap().polls.len() >= count {
                return;
            }
            assert!(
                tokio::time::Instant::now() < deadline,
                "gave up waiting for {count} zk-nym polls"
            );
            tokio::time::sleep(Duration::from_millis(25)).await;
        }
    }

    //---------------------
    // Route handlers
    //---------------------

    fn post_zknym(self) -> impl Respond {
        move |req: &Request| {
            let request: RequestZkNymRequestBody = req.body_json().unwrap();
            let id = Alphanumeric.sample_string(&mut rand::thread_rng(), 15);
            let response = post_response(&id, &request.ticketbook_type);

            let mut state = self.state.lock().unwrap();
            state.posts += 1;
            state.records.insert(
                id,
                ZkNymRecord {
                    status: NymVpnZkNymStatus::Pending,
                    request,
                },
            );

            ResponseTemplate::new(200).set_body_json(response)
        }
    }

    fn poll_zknym(self) -> impl Respond {
        move |req: &Request| {
            let id = req.url.path_segments().unwrap().next_back().unwrap();

            let mut state = self.state.lock().unwrap();
            state.polls.push(id.to_string());
            if state.activate_on_next_poll {
                state.activate_on_next_poll = false;
                for record in state.records.values_mut() {
                    if record.status == NymVpnZkNymStatus::Pending {
                        record.status = NymVpnZkNymStatus::Active;
                    }
                }
            }

            let Some(record) = state.records.get(id) else {
                return ResponseTemplate::new(404);
            };
            let shares = (record.status == NymVpnZkNymStatus::Active).then(|| {
                self.issue_wallet_shares(clone_request(&record.request))
                    .unwrap()
            });
            ResponseTemplate::new(200).set_body_json(zknym_response(
                id,
                &record.request.ticketbook_type,
                record.status.clone(),
                shares,
            ))
        }
    }

    fn list_zknyms(self) -> impl Respond {
        move |_req: &Request| {
            let state = self.state.lock().unwrap();
            let items = state
                .records
                .iter()
                .map(|(id, record)| {
                    zknym_response(
                        id,
                        &record.request.ticketbook_type,
                        record.status.clone(),
                        None,
                    )
                })
                .collect::<Vec<_>>();
            ResponseTemplate::new(200).set_body_json(listing(items))
        }
    }

    fn list_available_zknyms(self) -> impl Respond {
        move |_req: &Request| {
            let state = self.state.lock().unwrap();
            let items = state
                .records
                .iter()
                .filter(|(_, record)| record.status == NymVpnZkNymStatus::Active)
                .map(|(id, record)| {
                    zknym_response(
                        id,
                        &record.request.ticketbook_type,
                        record.status.clone(),
                        None,
                    )
                })
                .collect::<Vec<_>>();
            ResponseTemplate::new(200).set_body_json(listing(items))
        }
    }

    //---------------------
    // Crypto helpers
    //---------------------

    fn issue_wallet_shares(
        &self,
        request: RequestZkNymRequestBody,
    ) -> anyhow::Result<TicketbookWalletSharesResponse> {
        let user_key = PublicKeyUser::from_base58_string(request.ecash_pubkey)?;
        let format = format_description!("[year]-[month]-[day]");
        let expiration_date =
            Date::parse(&request.expiration_date, &format)?.ecash_unix_timestamp();
        let t_type = TicketType::from_str(&request.ticketbook_type)?.encode();
        let req = WithdrawalRequest::try_from_bs58(request.withdrawal_request)?;
        let mut shares = Vec::new();
        for auth_keypair in &self.authorities_keypairs {
            let blind_signature = issue(
                auth_keypair.secret_key(),
                user_key,
                &req,
                expiration_date,
                t_type,
            )?;
            shares.push(WalletShare {
                node_index: auth_keypair.index.unwrap(),
                bs58_encoded_share: blind_signature.to_bs58(),
            });
        }

        Ok(TicketbookWalletSharesResponse {
            epoch_id: 0,
            shares,
            master_verification_key: Some(MasterVerificationKeyResponse {
                epoch_id: 0,
                bs58_encoded_key: self.master_key.to_bs58(),
            }),
            aggregated_coin_index_signatures: Some(AggregatedCoinIndicesSignaturesResponse {
                signatures: AggregatedCoinIndicesSignatures {
                    epoch_id: 0,
                    signatures: self.coin_indices_signatures.clone(),
                },
            }),
            aggregated_expiration_date_signatures: Some(
                AggregatedExpirationDateSignaturesResponse {
                    signatures: AggregatedExpirationDateSignatures {
                        epoch_id: 0,
                        signatures: self.expiration_date_signatures.clone(),
                        expiration_date: nym_ecash_time::cred_exp_date().ecash_date(),
                    },
                },
            ),
        })
    }

    fn partial_verification_keys(&self) -> PartialVerificationKeysResponse {
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
}

// RequestZkNymRequestBody does not derive Clone
fn clone_request(request: &RequestZkNymRequestBody) -> RequestZkNymRequestBody {
    RequestZkNymRequestBody {
        withdrawal_request: request.withdrawal_request.clone(),
        ecash_pubkey: request.ecash_pubkey.clone(),
        expiration_date: request.expiration_date.clone(),
        ticketbook_type: request.ticketbook_type.clone(),
    }
}

fn post_response(id: &str, ticketbook_type: &str) -> NymVpnZkNymPost {
    let now = OffsetDateTime::now_utc();
    NymVpnZkNymPost {
        created_on_utc: now.to_string(),
        last_updated_utc: now.to_string(),
        id: id.to_string(),
        ticketbook_type: ticketbook_type.to_string(),
        valid_until_utc: (now + Duration::from_secs(3600 * 24 * 30)).to_string(),
        valid_from_utc: now.to_string(),
        issued_bandwidth_in_gb: 25f64,
        blinded_shares: None,
        status: NymVpnZkNymStatus::Pending,
    }
}

fn zknym_response(
    id: &str,
    ticketbook_type: &str,
    status: NymVpnZkNymStatus,
    blinded_shares: Option<TicketbookWalletSharesResponse>,
) -> NymVpnZkNym {
    let now = OffsetDateTime::now_utc();
    NymVpnZkNym {
        created_on_utc: now.to_string(),
        last_updated_utc: now.to_string(),
        id: id.to_string(),
        ticketbook_type: ticketbook_type.to_string(),
        valid_until_utc: (now + Duration::from_secs(3600 * 24 * 30)).to_string(),
        valid_from_utc: now.to_string(),
        issued_bandwidth_in_gb: 25f64,
        blinded_shares,
        status,
        upgrade_mode: None,
    }
}

fn listing(items: Vec<NymVpnZkNym>) -> NymVpnZkNymResponse {
    NymVpnZkNymResponse {
        total_items: items.len() as u64,
        page: 0,
        page_size: items.len().max(1) as u64,
        items,
    }
}

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
