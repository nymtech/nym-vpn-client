// Copyright 2026 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{
    borrow::Cow,
    path::{Path, PathBuf},
};

use base64::{Engine, engine::general_purpose::STANDARD};
use serde::{Deserialize, Serialize};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::UnixStream,
};
use zeroize::{Zeroize, Zeroizing};

use crate::{Result, SecureEnclaveError};

/// Socket of the varlink service of `systemd-creds`, available since systemd 256.
const SOCKET_PATH: &str = "/run/systemd/io.systemd.Credentials";

const ENCRYPT_METHOD: &str = "io.systemd.Credentials.Encrypt";
const DECRYPT_METHOD: &str = "io.systemd.Credentials.Decrypt";

/// Encrypted credentials are bound to the system rather than to a user.
const SYSTEM_SCOPE: &str = "system";

/// Binds the credentials to both the TPM and the host key when the TPM is available, otherwise to
/// the host key only.
const AUTO_KEY: &str = "auto";

/// Room for the varlink message around the parameters.
const MESSAGE_OVERHEAD: usize = 256;

/// Client of the `systemd-creds` varlink service, which encrypts data with the TPM and the host
/// key held by systemd.
#[derive(Clone)]
pub struct SystemdCredentials {
    socket_path: PathBuf,
}

impl SystemdCredentials {
    /// Returns `None` when the service is not available on this system.
    pub fn new() -> Option<Self> {
        Path::new(SOCKET_PATH).exists().then(|| Self {
            socket_path: PathBuf::from(SOCKET_PATH),
        })
    }

    /// Encrypts the data into a credential authenticated with the `name`. The credential is
    /// Base64 encoded.
    pub async fn encrypt(&self, name: &str, data: &[u8]) -> Result<Vec<u8>> {
        let data = Zeroizing::new(STANDARD.encode(data));
        let parameters = EncryptParameters {
            name,
            data: &data,
            scope: SYSTEM_SCOPE,
            with_key: AUTO_KEY,
        };
        let reply = self.call(ENCRYPT_METHOD, &parameters, data.len()).await?;
        let reply: Reply<EncryptReply<'_>> = parse_reply(&reply)?;

        Ok(reply.parameters.blob.as_bytes().to_owned())
    }

    /// Decrypts the credential, which must be authenticated with the same `name` it was encrypted
    /// with.
    pub async fn decrypt(&self, name: &str, blob: &[u8]) -> Result<Vec<u8>> {
        let blob = std::str::from_utf8(blob)
            .map_err(|_| SecureEnclaveError::Other("invalid credential".to_owned()))?;
        let parameters = DecryptParameters {
            name,
            blob,
            scope: SYSTEM_SCOPE,
        };
        let reply = self.call(DECRYPT_METHOD, &parameters, blob.len()).await?;
        let reply: Reply<DecryptReply<'_>> = parse_reply(&reply)?;

        let mut data = reply.parameters.data;
        let decoded = STANDARD
            .decode(data.as_bytes())
            .map_err(|_| SecureEnclaveError::Other("invalid decrypted data".to_owned()));
        if let Cow::Owned(data) = &mut data {
            data.zeroize();
        }
        decoded
    }

    /// Calls the method and returns the reply without the terminating NUL. The request and the
    /// reply may contain secrets, so their buffers are zeroed once no longer used.
    async fn call<P: Serialize>(
        &self,
        method: &str,
        parameters: &P,
        size_hint: usize,
    ) -> Result<Zeroizing<Vec<u8>>> {
        let mut stream = UnixStream::connect(&self.socket_path)
            .await
            .map_err(|err| io_error("connect to systemd-creds", err))?;

        let mut request = Zeroizing::new(Vec::with_capacity(size_hint + MESSAGE_OVERHEAD));
        serde_json::to_writer(&mut *request, &Request { method, parameters })
            .map_err(|err| SecureEnclaveError::Other(format!("failed to encode request: {err}")))?;
        request.push(0);
        stream
            .write_all(&request)
            .await
            .map_err(|err| io_error("send request to systemd-creds", err))?;

        let mut reply = Zeroizing::new(Vec::with_capacity(size_hint + MESSAGE_OVERHEAD));
        while reply.last() != Some(&0) {
            if reply.len() == reply.capacity() {
                reply = grow(&reply);
            }
            let read = stream
                .read_buf(&mut *reply)
                .await
                .map_err(|err| io_error("read reply from systemd-creds", err))?;
            if read == 0 {
                return Err(SecureEnclaveError::Other(
                    "systemd-creds closed the connection".to_owned(),
                ));
            }
        }
        reply.pop();

        Ok(reply)
    }
}

#[derive(Serialize)]
struct Request<'a, P> {
    method: &'a str,
    parameters: &'a P,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct EncryptParameters<'a> {
    name: &'a str,
    data: &'a str,
    scope: &'a str,
    with_key: &'a str,
}

#[derive(Serialize)]
struct DecryptParameters<'a> {
    name: &'a str,
    blob: &'a str,
    scope: &'a str,
}

#[derive(Deserialize)]
struct ErrorReply {
    error: Option<String>,
}

#[derive(Deserialize)]
struct Reply<P> {
    parameters: P,
}

#[derive(Deserialize)]
struct EncryptReply<'a> {
    #[serde(borrow)]
    blob: Cow<'a, str>,
}

/// The data is borrowed from the reply buffer, unless it contains escape sequences.
#[derive(Deserialize)]
struct DecryptReply<'a> {
    #[serde(borrow)]
    data: Cow<'a, str>,
}

fn parse_reply<'a, P>(reply: &'a [u8]) -> Result<Reply<P>>
where
    Reply<P>: Deserialize<'a>,
{
    let error = serde_json::from_slice::<ErrorReply>(reply)
        .map_err(invalid_reply)?
        .error;
    match error {
        Some(error) => Err(SecureEnclaveError::Other(format!(
            "systemd-creds failed: {error}"
        ))),
        None => serde_json::from_slice(reply).map_err(invalid_reply),
    }
}

/// Copies the data into a buffer with twice the capacity, so that no copy of it is left behind
/// unzeroed by reallocation.
fn grow(data: &[u8]) -> Zeroizing<Vec<u8>> {
    let mut grown = Zeroizing::new(Vec::with_capacity(data.len().max(1) * 2));
    grown.extend_from_slice(data);
    grown
}

fn invalid_reply(err: serde_json::Error) -> SecureEnclaveError {
    SecureEnclaveError::Other(format!("invalid reply from systemd-creds: {err}"))
}

fn io_error(ctx: &str, err: std::io::Error) -> SecureEnclaveError {
    SecureEnclaveError::Other(format!("failed to {ctx}: {err}"))
}
