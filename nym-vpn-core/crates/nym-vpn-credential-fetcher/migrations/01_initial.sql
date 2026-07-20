/*
 * Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
 * SPDX-License-Identifier: GPL-3.0-only
 */

CREATE TABLE pending_zk_nym_requests
(
    id                  TEXT        NOT NULL PRIMARY KEY,
    expiration_date     TEXT        NOT NULL,
    request_info        BLOB        NOT NULL,
    timestamp           TIMESTAMP   WITHOUT TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP
);
