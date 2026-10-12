// Copyright 2026 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use zeroize::{Zeroize, Zeroizing};

use crate::{Result, SecureEnclaveError, SecureEnclaveKey, SecureEnclaveProvider};

#[derive(Default)]
pub struct AppleSecureEnclaveProvider;

#[async_trait::async_trait]
impl SecureEnclaveProvider for AppleSecureEnclaveProvider {
    type Key = OSSecureEnclaveKey;

    async fn create_key(&self, alias: String) -> Result<Self::Key> {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<SecureEnclaveOperation>();

        spawn_worker_thread(rx, move || {
            secure_enclave_rs::SecureEnclaveKey::generate(
                &secure_enclave_rs::SecureEnclaveKeyOptions {
                    tag: alias.as_bytes(),
                    access_flags: secure_enclave_rs::SecAccessControlFlags::USER_PRESENCE,
                    permanent: true,
                },
            )
            .map_err(SecureEnclaveError::from)
        })
        .await?;

        Ok(OSSecureEnclaveKey::new(tx))
    }

    async fn delete_key(&self, alias: String) -> Result<()> {
        secure_enclave_rs::SecureEnclaveKey::remove_by_tag(alias.as_bytes())?;
        Ok(())
    }

    async fn get_key(&self, alias: String) -> Result<Self::Key> {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<SecureEnclaveOperation>();

        spawn_worker_thread(rx, move || {
            secure_enclave_rs::SecureEnclaveKey::get(alias.as_bytes())
                .map_err(SecureEnclaveError::from)
        })
        .await?;

        Ok(OSSecureEnclaveKey::new(tx))
    }
}

#[derive(Clone)]
pub struct OSSecureEnclaveKey {
    tx: tokio::sync::mpsc::UnboundedSender<SecureEnclaveOperation>,
}

impl OSSecureEnclaveKey {
    fn new(tx: tokio::sync::mpsc::UnboundedSender<SecureEnclaveOperation>) -> Self {
        Self { tx }
    }
}

#[async_trait::async_trait]
impl SecureEnclaveKey for OSSecureEnclaveKey {
    async fn decrypt(&self, bytes: Vec<u8>) -> Result<Vec<u8>> {
        let (tx, rx) = tokio::sync::oneshot::channel();
        let _ = self.tx.send(SecureEnclaveOperation::Decrypt {
            bytes,
            completion: tx,
        });
        rx.await
            .map_err(|_| SecureEnclaveError::Other("channel closed".to_owned()))?
    }

    async fn encrypt(&self, bytes: Vec<u8>) -> Result<Vec<u8>> {
        let (tx, rx) = tokio::sync::oneshot::channel();
        let _ = self.tx.send(SecureEnclaveOperation::Encrypt {
            bytes: Zeroizing::new(bytes),
            completion: tx,
        });
        rx.await
            .map_err(|_| SecureEnclaveError::Other("channel closed".to_owned()))?
    }
}

impl From<secure_enclave_rs::Error> for SecureEnclaveError {
    fn from(error: secure_enclave_rs::Error) -> Self {
        match error {
            secure_enclave_rs::Error::AuthFailed => Self::AuthenticationFailed,
            secure_enclave_rs::Error::NotFound => Self::KeyNotFound,
            other => Self::Other(other.to_string()),
        }
    }
}

enum SecureEnclaveOperation {
    Decrypt {
        bytes: Vec<u8>,
        completion: tokio::sync::oneshot::Sender<Result<Vec<u8>>>,
    },
    Encrypt {
        bytes: Zeroizing<Vec<u8>>,
        completion: tokio::sync::oneshot::Sender<Result<Vec<u8>>>,
    },
}

/// Spawn a new thread binding `secure_enclave_rs::SecureEnclaveKey` key to it and establishing communication using channels
async fn spawn_worker_thread<F>(
    mut command_rx: tokio::sync::mpsc::UnboundedReceiver<SecureEnclaveOperation>,
    key_fn: F,
) -> Result<()>
where
    F: FnOnce() -> Result<secure_enclave_rs::SecureEnclaveKey>,
    F: Send + 'static,
{
    let (init_tx, init_rx) = tokio::sync::oneshot::channel();

    let _join_handle = std::thread::spawn(move || match key_fn() {
        Ok(key) => {
            let _ = init_tx.send(Ok(()));

            while let Some(op) = command_rx.blocking_recv() {
                match op {
                    SecureEnclaveOperation::Decrypt { bytes, completion } => {
                        let res = key.decrypt(&bytes);
                        if let Err(Ok(mut plaintext)) =
                            completion.send(res.map_err(SecureEnclaveError::from))
                        {
                            plaintext.zeroize();
                        }
                    }
                    SecureEnclaveOperation::Encrypt { bytes, completion } => {
                        let res = key.encrypt(&bytes);
                        let _ = completion.send(res.map_err(SecureEnclaveError::from));
                    }
                }
            }
        }
        Err(err) => {
            let _ = init_tx.send(Err(err));
            return;
        }
    });

    init_rx
        .await
        .map_err(|_| SecureEnclaveError::Other("channel closed".to_owned()))?
}
