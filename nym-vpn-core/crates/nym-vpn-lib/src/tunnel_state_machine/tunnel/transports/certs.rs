// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

//! Certificate handling implementations
//!
//! Nym Nodes have identity keys (ed25519) that can be used to sign and verify certificates to use
//! with QUIC TLS.
//!
//! The server:
//! - has the ed25519 private key that all nodes have
//! - uses this key to create a self-signed certificate
//!   - sets the certificates common name to the base58 encoded identity key
//!
//! The client:
//! - implements a custom certificate verifier that:
//!   - only accepts ed25519 signatures
//!   - "hostname" / SNI check is set to "base58 encoded identity key" or any configured domain name
//!      - if the Server Name is not in the acceptable configured alt-names it throws a
//!        NotValidForName error
//!   - at least one common name entry is either the "base58 encoded identity key" or any configured
//!     domain name
//!     - if none of the common names are in the acceptable configured alt-names it throws a
//!       NotValidForName error
//!   - the public key from the certificate PKI is the server's ed25519 identity key
//!   - cert is signed correctly
//!     - done last to avoid ed25519 signature verification in case string based checks fail
//!   - uses the default [`rustls::client::WebPkiServerVerifier`] to verify TLS 1.2 / TLS 1.3

use ed25519_dalek::{VerifyingKey, pkcs8::DecodePublicKey};
use rustls::{
    CertificateError, DigitallySignedStruct, RootCertStore, SignatureScheme,
    client::{
        WebPkiServerVerifier,
        danger::{HandshakeSignatureValid, ServerCertVerified, ServerCertVerifier},
    },
    pki_types::{CertificateDer, ServerName, UnixTime},
    server::VerifierBuilderError,
};
use tracing::*;
use webpki_roots::TLS_SERVER_ROOTS;
use x509_parser::prelude::*;

use std::sync::Arc;

fn parse_certificate(
    cert_der: &[u8],
) -> Result<X509Certificate<'_>, x509_parser::nom::Err<X509Error>> {
    let res = X509Certificate::from_der(cert_der)?;
    Ok(res.1)
}

#[derive(Debug)]
pub struct IdentityBasedVerifierBuilder {
    alt_names: Vec<String>,
    identity_pubkey: VerifyingKey,
    crypto_provider: Option<Arc<rustls::crypto::CryptoProvider>>,
}

impl IdentityBasedVerifierBuilder {
    pub fn build(self) -> Result<IdentityBasedVerifier, VerifierBuilderError> {
        let pubkey_as_name = bs58::encode(self.identity_pubkey.as_bytes()).into_string();
        trace!("building identity key based verified with key: {pubkey_as_name}");
        // ensure that the alt names contains the public key as a string
        let mut alt_names = self.alt_names;
        if !alt_names.contains(&pubkey_as_name) {
            alt_names.push(pubkey_as_name);
        }

        // create an empty trust store
        let mut roots = RootCertStore::empty();

        // annoyingly rustls wants CA root certificates - might be possible to set a single fake one to keep it happy
        roots.extend(TLS_SERVER_ROOTS.iter().cloned());

        // create a verifier so we can use default implementations
        let default_verifier = if let Some(crypto_provider) = self.crypto_provider {
            WebPkiServerVerifier::builder_with_provider(Arc::new(roots), crypto_provider).build()?
        } else {
            WebPkiServerVerifier::builder(Arc::new(roots)).build()?
        };

        Ok(IdentityBasedVerifier {
            alt_names,
            server_identity_pubkey: self.identity_pubkey.to_bytes(),
            default_verifier,
        })
    }

    /// Appends alt names to the set accepted for this server certificate verifier.
    /// Can be called multiple times as provided names are appended.
    pub fn with_alt_names(mut self, alt_names: Option<Vec<impl ToString>>) -> Self {
        let mut alt_names: Vec<String> = alt_names
            .unwrap_or_default()
            .iter()
            .map(ToString::to_string)
            .filter(|x| !self.alt_names.contains(x))
            .collect();

        self.alt_names.append(&mut alt_names);
        self
    }

    /// Allows a caller to set a custom rustls CryptoProvider. If none is given
    /// the default construction is used which assumes that a default CryptoProvider
    /// has been installed at the crate level.
    pub fn with_crypto_provider(
        mut self,
        crypto_provider: Arc<rustls::crypto::CryptoProvider>,
    ) -> Self {
        self.crypto_provider = Some(crypto_provider);
        self
    }
}

#[derive(Debug)]
pub struct IdentityBasedVerifier {
    alt_names: Vec<String>,
    server_identity_pubkey: [u8; ed25519_dalek::PUBLIC_KEY_LENGTH],
    default_verifier: Arc<WebPkiServerVerifier>,
}

impl IdentityBasedVerifier {
    pub fn builder(identity_key: &VerifyingKey) -> IdentityBasedVerifierBuilder {
        IdentityBasedVerifierBuilder {
            alt_names: vec![],
            identity_pubkey: *identity_key,
            crypto_provider: None,
        }
    }
}

impl ServerCertVerifier for IdentityBasedVerifier {
    fn verify_server_cert(
        &self,
        end_entity: &CertificateDer,
        intermediates: &[CertificateDer],
        server_name: &ServerName,
        ocsp_response: &[u8],
        now: UnixTime,
    ) -> Result<ServerCertVerified, quinn::rustls::Error> {
        trace!(
            ">>>> verify_server_cert: end_entity: {end_entity:?}, intermediates: {intermediates:?}, server_name: {server_name:?}, ocsp_response: {ocsp_response:?}, now: {now:?}",
        );

        // check Server Name against the configured acceptable names
        if !self.alt_names.contains(&server_name.to_str().to_string()) {
            trace!(
                "⛔️ unexpected server name: {:?} {:?}",
                server_name, self.alt_names
            );
            return Err(quinn::rustls::Error::InvalidCertificate(
                CertificateError::NotValidForName,
            ));
        }

        match parse_certificate(end_entity) {
            Ok(cert) => {
                // check Common Name against the configured acceptable names
                if cert.subject.iter_common_name().any(|cn| {
                    cn.as_str().is_ok_and(|v| {
                        trace!("  - CN = {v}");
                        self.alt_names.contains(&v.to_string())
                    })
                }) {
                    trace!("✅ acceptable common name");
                } else {
                    trace!("⛔️ unexpected common name");
                    return Err(quinn::rustls::Error::InvalidCertificate(
                        CertificateError::NotValidForName,
                    ));
                }

                // extract the public key
                let public_key = cert.public_key();
                let raw_public_key = public_key.raw;

                trace!("public key in certificate: {public_key:?}, bytes: {raw_public_key:?}");

                // check that the public key associated with the cert matches the identity key of the server
                match ed25519_dalek::pkcs8::PublicKeyBytes::from_public_key_der(raw_public_key) {
                    Ok(pk) => {
                        if let Ok(vk) = ed25519_dalek::VerifyingKey::from_bytes(&pk.to_bytes()) {
                            if vk.to_bytes() == self.server_identity_pubkey {
                                trace!("✅ parsed public key in certificate matches identity key");
                            } else {
                                trace!(
                                    "😢 parsed public key in certificate does not match identity key"
                                );
                                return Err(quinn::rustls::Error::InvalidCertificate(
                                    CertificateError::NotValidForName,
                                ));
                            }
                        } else {
                            trace!(
                                "😢 Could not decode subject public key into a ed25519 public key"
                            );
                        }
                    }
                    Err(_) => {
                        trace!("😢 Could not parse subject public key info from certificate");
                    }
                }

                // the certificate is self-signed, so let it verify itself
                //
                // done last in case cheaper string matching checks fail
                match cert.verify_signature(None) {
                    Ok(()) => {
                        trace!(
                            "✅ self signed certificate is valid using certificate's own public key info"
                        );
                    }
                    Err(_) => {
                        trace!("⛔️ self signed certificate failed to verify signature");
                        return Err(quinn::rustls::Error::InvalidCertificate(
                            CertificateError::BadEncoding,
                        ));
                    }
                }
            }
            Err(_) => {
                return Err(quinn::rustls::Error::InvalidCertificate(
                    CertificateError::BadEncoding,
                ));
            }
        }

        trace!("🌈 everything looks good");
        Ok(ServerCertVerified::assertion())
    }

    fn verify_tls12_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, quinn::rustls::Error> {
        trace!(">>>> verify_tls12_signature: message: {message:?}, cert: {cert:?}, dss: {dss:?}");
        self.default_verifier
            .verify_tls12_signature(message, cert, dss)
    }

    fn verify_tls13_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, quinn::rustls::Error> {
        trace!(">>>> verify_tls13_signature: message: {message:?}, cert: {cert:?}, dss: {dss:?}");
        self.default_verifier
            .verify_tls13_signature(message, cert, dss)
    }

    fn supported_verify_schemes(&self) -> Vec<SignatureScheme> {
        vec![SignatureScheme::ED25519]
    }
}
