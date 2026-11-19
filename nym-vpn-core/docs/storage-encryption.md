# Storage encryption

This document describes how `nym-vpn-core` protects data stored on the device: the account
mnemonic, account information, device keys, databases and private key files.

The relevant code lives in:

- `crates/nym-secure-enclave` – platform key stores (Keychain, Android Keystore, TPM, `systemd-creds`).
- `crates/nym-vpn-store` – master key, store ciphers and the individual stores.
- `crates/nym-vpn-lib/src/storage` – per-platform wiring of the stores.

## Overview

```
                    platform key store
          (Keychain / Keystore / TPM / systemd-creds)
                            │
                            ▼
                       master key (32 bytes, random)
                            │  HKDF-SHA256
     ┌───────────────┬──────┴────────┬──────────────────────┐
     ▼               ▼               ▼                      ▼
 database key    key secret     account summary key    account info key
 (SQLCipher)     (PEM files)    (AES-256-GCM)           (AES-256-GCM)
```

A single random **master key** is created on first start and stored in the platform key store.
Every other key is derived from it, so the platform key store only ever protects one or two small
secrets. The mnemonic is not protected by the master key: it is encrypted separately so that it
can require user presence where the platform supports it.

## Master key

- 32 random bytes, generated with the OS RNG (`MasterKey::generate`).
- Kept on the heap and zeroized on drop.
- Loaded or created once in `VpnClientOnDiskStorage::new`. Concurrent creation by two processes
  is resolved by creating the file with `create_new` and loading the winner's key on
  `AlreadyExists`.
- On disk (`master.key`) it is written with `0600` permissions on Unix.

### Derived keys

Keys are derived with HKDF-SHA256 (no salt, master key as IKM), using a distinct label for each
purpose so that ciphertext produced for one purpose can't be used for another:

| Key | Label | Used for |
|---|---|---|
| Database key | `net.nymtech.vpn.database-key.v1` | SQLCipher databases |
| Key secret | `net.nymtech.vpn.pem-key-secret.v1` | Private key PEM files |
| Account summary key | `net.nymtech.vpn.account-summary-key.v1` | `account_summary.bin` |
| Account info key | `net.nymtech.vpn.account-info-key.v1` | `account.bin` |

## Ciphers

### AES-GCM store cipher

`AesGcmStoreCipher` encrypts small files with AES-256-GCM:

```
version (1 byte, = 1) ‖ nonce (12 bytes, random) ‖ ciphertext ‖ tag (16 bytes)
```

### Secure enclave cipher

`SecureEnclaveCipher` delegates encryption to a key held by the platform key store, identified by
an alias. The key is created on first use (`SecureEnclave::get_or_create_key`) and cached for the
lifetime of the cipher. The ciphertext format depends on the platform, see below.

### SQLCipher

Databases are opened through `nym_sqlcipher::apply_key`. The database key is passed as a raw key
(`PRAGMA key = "x'<hex>'"`), so SQLCipher uses it directly without passphrase key derivation.
A plaintext database found at the path is encrypted first, by exporting it into an encrypted copy
which then replaces the original.

Encrypted databases:

- ecash credential storage
- pending credential requests (`pending_credential_requests.db`)
- WireGuard keys
- statistics
- mixnet client storage (VPN and SOCKS5 clients)

### PEM files

Private keys (device keys, mixnet client keys) are stored as `ENCRYPTED <TYPE>` PEM files by
`nym-pemstore`, using PBES2 with AES-256-GCM. The AES key is derived from the key secret with
PBKDF2-HMAC-SHA256 using a single iteration and a random 128-bit salt per file. A single iteration
is enough because the key secret is already a uniformly random 256-bit key. Private keys stored in
plaintext before encryption was introduced are re-stored encrypted when loaded.

## Platforms

| Platform | Master key | Mnemonic | Account info | User presence for mnemonic | Fallback |
|---|---|---|---|---|---|
| iOS | Keychain item, available after first unlock | Own keychain item, requires user presence and an unlocked device | Keychain item, available after first unlock | Yes | — |
| macOS | Keychain item, available after first unlock | Own keychain item, available after first unlock | Keychain item, available after first unlock | No | — |
| Android | `master.key`, encrypted by Keystore key `device-key` | `mnemonic.bin`, encrypted by Keystore key `mnemonic-key` | `account.bin`, account info key | Yes | TEE-backed Keystore without StrongBox |
| Windows | `master.key`, encrypted by TPM key `net.nymtech.vpn.device-key` | `mnemonic.bin`, encrypted by the same TPM key as the master key | `account.bin`, account info key | No | DPAPI |
| Linux | `master.key`, encrypted by `systemd-creds` as `net.nymtech.vpn.device-key` | `mnemonic.bin`, encrypted by the same credential as the master key | `account.bin`, account info key | No | Plaintext |
| Other | `master.key`, plaintext | `mnemonic.bin`, plaintext | `account.bin`, plaintext | No | — |

On every platform the account summary (`account_summary.bin`), databases and PEM files are
encrypted with keys derived from the master key.

### iOS and macOS

Secrets are stored as generic password items in the data protection keychain
(`kSecUseDataProtectionKeychain`). Items are `ThisDeviceOnly`, so they are never synced or included
in backups. Each item is identified by a service and the network name as the account:

| Service | Contents |
|---|---|
| `net.nymtech.vpn.master-key` | Master key |
| `net.nymtech.vpn.mnemonic` | Mnemonic |
| `net.nymtech.vpn.account-info` | Account information |
| `net.nymtech.vpn.device-keys` | Device keys |

On Apple platforms account information and device keys are stored in the keychain directly rather
than in files. Device keys found on disk in plaintext are moved to the keychain.

- Items other than the mnemonic use `kSecAttrAccessibleAfterFirstUnlockThisDeviceOnly`, so that
  the tunnel can start while the device is locked.
- On iOS the mnemonic uses `kSecAttrAccessibleWhenUnlockedThisDeviceOnly` with a
  `UserPresence` access control. On macOS `nym-vpnd` runs as root and can't prompt the user, so
  the mnemonic is available after first unlock like the other items.
- On iOS the items are in the `group.net.nymtech.vpn` access group, shared by the app and the
  network extension. On macOS they are in the default access group of the daemon.

User presence is handled by `LocalAuthentication`. A single `LAContext` is evaluated on
`unlock()` and used for all keychain operations on a dedicated worker thread until `lock()`
replaces it with a non-interactive context. While locked, items requiring user presence fail with
`Locked` instead of prompting.

On macOS, `nym-vpnd` registers an authorization right used to authenticate the user requesting
the unlock.

The worker thread installs a CoreFoundation default allocator that zeroes memory before freeing
it, so CF objects created by the Security framework on that thread, such as returned item data,
are wiped on release.

### Android

Keys are AES-256-GCM keys generated in the Android Keystore (`KeystoreSecureEnclaveProvider` in
the app), exposed to Rust through uniffi. StrongBox is used when available, otherwise the TEE.
Key material never leaves the Keystore.

Ciphertext format: `IV (12 bytes) ‖ ciphertext ‖ tag (16 bytes)`.

| Alias | Encrypts | User authentication |
|---|---|---|
| `device-key` | `master.key` | Not required |
| `mnemonic-key` | `mnemonic.bin` | Required |

`mnemonic-key` requires strong biometrics or the device credential, stays usable for 300 seconds
after authentication, requires an unlocked device (API 28+) and is not invalidated by enrolling new
biometrics.

The Keystore can't revoke an authentication window, so `SecureEnclaveSession` tracks the lock state
in the app: while locked, keys requiring user presence fail with `Locked`. `unlock()` prompts at
most once for concurrent callers and does nothing while the authentication window is still open.

A key invalidated by removing the secure lock screen is deleted and reported as `KeyNotFound`.

### Windows

Keys are created by the Platform Crypto Provider (`MS_PLATFORM_CRYPTO_PROVIDER`), which stores
them in the TPM. Keys belong to the current user, which is `SYSTEM` for the daemon.

- **TPM**: a non-exportable RSA-2048 key. Each encryption uses a fresh AES-256-GCM key, wrapped
  with the TPM key using RSA-OAEP with SHA-256.

  ```
  tag (1 byte, = 1) ‖ wrapped key length (u16 LE) ‖ wrapped key ‖ nonce (12 bytes) ‖ ciphertext ‖ tag
  ```

- **DPAPI**: used when the machine has no usable TPM. Data is encrypted with `CryptProtectData`
  for the current user, with the key alias as additional entropy.

  ```
  tag (1 byte, = 2) ‖ DPAPI blob
  ```

The leading tag selects the backend on decryption, so data encrypted with DPAPI remains readable
after a TPM key is created.

User presence is not supported. `unlock()` and `lock()` do nothing.

### Linux

Data is encrypted by the `systemd-creds` varlink service (`/run/systemd/io.systemd.Credentials`,
systemd 256 or later) in the system scope. With the `auto` key mode, credentials are bound to both
the TPM and the host key held by systemd when a TPM is available, otherwise to the host key only.
The key alias is used as the credential name, which is authenticated on decryption. Decryption
requires root.

```
tag (1 byte) ‖ data
```

| Tag | Data |
|---|---|
| 1 | Base64 encoded `systemd-creds` credential |
| 2 | Plaintext, written when `systemd-creds` is unavailable |

There are no keys to create or delete. User presence is not supported. `unlock()` and `lock()` do
nothing.

## Migration from plaintext storage

On startup `VpnClientOnDiskStorage::new` migrates data stored before encryption was introduced:

- `mnemonic.json` is moved to the mnemonic store and account storage
  (`SecureStoreMigrationManager`). On Android the user is asked to unlock first. On failure the
  plaintext file is kept and the migration is retried on the next start.
- `account_summary.json` is replaced by the encrypted `account_summary.bin`.
- Plaintext SQLite databases are encrypted in place when opened.
- Plaintext private key PEM files are re-stored encrypted when loaded.
- On Apple platforms, device keys found on disk are moved to the keychain.

## Known limitations

- On Linux without `systemd-creds` the master key and mnemonic are stored in plaintext, protected
  only by file permissions. Only a warning is logged.
- On Windows and Linux the mnemonic has the same protection as the master key. Anyone able to
  decrypt the master key (`SYSTEM` or root) can decrypt the mnemonic without prompting the user.
- On macOS the mnemonic does not require user presence.
- Copies of secrets held by system services (`securityd`, Keystore, TPM stack, `systemd`) and in
  IPC buffers are outside the client's control.
