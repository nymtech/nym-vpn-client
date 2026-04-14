// Copyright 2016-2026 Mullvad VPN AB. All Rights Reserved.
// Copyright 2026 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

#pragma once

//
// types.h
//
// Miscellaneous types and defines used internally.
//

#define ST_POOL_TAG 'UTPS'

enum class ST_PAGEABLE {
    YES = 0,
    NO
};

//
// Type-safety when passing around lower case device paths.
// Same definition as UNICODE_STRING so they can be cast between.
//
typedef struct tag_LOWER_UNICODE_STRING {
    USHORT Length;
    USHORT MaximumLength;
    PWCH Buffer;
} LOWER_UNICODE_STRING;

enum ST_PROCESS_SPLIT_STATUS {
    // Traffic should be split (excluded from VPN).
    ST_PROCESS_SPLIT_STATUS_ON_BY_CONFIG = 0,

    // Traffic should be split (excluded from VPN).
    ST_PROCESS_SPLIT_STATUS_ON_BY_INHERITANCE,

    // Traffic should not be split.
    ST_PROCESS_SPLIT_STATUS_OFF,

    // Traffic is routed according to the source IP address the process binds to.
    // No bind/connect rewriting is performed; the process is responsible for
    // binding to the correct interface. Non-tunnel traffic is permitted.
    ST_PROCESS_SPLIT_STATUS_HYBRID_BY_CONFIG
};
