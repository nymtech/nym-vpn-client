// Copyright 2021-2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: Apache-2.0

use crate::error::BandwidthControllerError;
use crate::utils::stored_credential_to_issued_bandwidth;
use log::{debug, error, warn};
use nym_credential_storage::storage::Storage;
use nym_credentials::coconut::bandwidth::issued::BandwidthCredentialIssuedDataVariant;
use nym_credentials::IssuedBandwidthCredential;

pub use event::BandwidthStatusMessage;

pub mod error;
mod event;
mod utils;

#[derive(Debug)]
pub struct BandwidthController<C, St> {
    storage: St,
    client: C,
}

pub struct RetrievedCredential {
    pub credential: IssuedBandwidthCredential,
    pub credential_id: i64,
}

impl<C, St: Storage> BandwidthController<C, St> {
    pub fn new(storage: St, client: C) -> Self {
        BandwidthController { storage, client }
    }

    /// Tries to retrieve one of the stored, unused credentials that hasn't yet expired.
    /// It marks any retrieved intermediate credentials as expired.
    pub async fn get_next_usable_credential(
        &self,
        gateway_id: &str,
    ) -> Result<RetrievedCredential, BandwidthControllerError>
    where
        <St as Storage>::StorageError: Send + Sync + 'static,
    {
        loop {
            let Some(maybe_next) = self
                .storage
                .get_next_unspent_credential(gateway_id)
                .await
                .map_err(|err| BandwidthControllerError::CredentialStorageError(Box::new(err)))?
            else {
                return Err(BandwidthControllerError::NoCredentialsAvailable);
            };
            let id = maybe_next.id;

            // try to deserialize it
            let valid_credential = match stored_credential_to_issued_bandwidth(maybe_next) {
                // check if it has already expired
                Ok(credential) => match credential.variant_data() {
                    BandwidthCredentialIssuedDataVariant::Voucher(_) => {
                        debug!("credential {id} is a bandwidth voucher");
                        credential
                    }
                    BandwidthCredentialIssuedDataVariant::FreePass(freepass_info) => {
                        debug!("credential {id} is a free pass");
                        if freepass_info.expired() {
                            warn!("the free pass (id: {id}) has already expired! The expiration was set to {}", freepass_info.expiry_date());
                            self.storage.mark_expired(id).await.map_err(|err| {
                                BandwidthControllerError::CredentialStorageError(Box::new(err))
                            })?;
                            continue;
                        }
                        credential
                    }
                },
                Err(err) => {
                    error!("failed to deserialize credential with id {id}: {err}. it may need to be manually removed from the storage");
                    return Err(err);
                }
            };
            return Ok(RetrievedCredential {
                credential: valid_credential,
                credential_id: id,
            });
        }
    }

    pub fn storage(&self) -> &St {
        &self.storage
    }
}

impl<C, St> Clone for BandwidthController<C, St>
where
    C: Clone,
    St: Clone,
{
    fn clone(&self) -> Self {
        BandwidthController {
            storage: self.storage.clone(),
            client: self.client.clone(),
        }
    }
}
