/*
 * Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
 * SPDX-License-Identifier: GPL-3.0-only
 */

CREATE TABLE wireguard_gateway_keys_new
(
    gateway_id_bs58         TEXT                        NOT NULL UNIQUE PRIMARY KEY,
    entry_private_key_bs58  TEXT                        NOT NULL,
    exit_private_key_bs58   TEXT                        NOT NULL,
    expiration_time         TIMESTAMP WITHOUT TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP
);

INSERT INTO wireguard_gateway_keys_new(gateway_id_bs58, entry_private_key_bs58, exit_private_key_bs58) SELECT gateway_id_bs58, entry_private_key_bs58, exit_private_key_bs58 FROM wireguard_gateway_keys;
DROP TABLE wireguard_gateway_keys;
ALTER TABLE wireguard_gateway_keys_new RENAME TO wireguard_gateway_keys;
