/*
 * Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
 * SPDX-License-Identifier: GPL-3.0-only
 */

CREATE TABLE wireguard_gateway_keys
(
    gateway_id_bs58         TEXT    NOT NULL UNIQUE PRIMARY KEY,
    entry_private_key_bs58  TEXT    NOT NULL,
    exit_private_key_bs58   TEXT    NOT NULL
);
