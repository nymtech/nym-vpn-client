// Copyright 2016-2026 Mullvad VPN AB. All Rights Reserved.
// Copyright 2026 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

#pragma once

//
// Structures related to configuration.
//

//
// Flags for ST_CONFIGURATION_ENTRY.
//

// Default: exclude the process from the VPN tunnel (route via internet interface).
#define ST_CONFIGURATION_ENTRY_FLAG_EXCLUDE 0x0000

// Route traffic according to the source IP the process binds to.
// No bind/connect rewriting is performed.
#define ST_CONFIGURATION_ENTRY_FLAG_HYBRID 0x0001

typedef struct tag_ST_CONFIGURATION_ENTRY {
    // Offset into buffer region that follows all entries.
    // The image name uses the device path.
    SIZE_T ImageNameOffset;

    // Byte length for non-null terminated wide char string.
    USHORT ImageNameLength;

    // Combination of ST_CONFIGURATION_ENTRY_FLAG_* values.
    USHORT Flags;
} ST_CONFIGURATION_ENTRY;

typedef struct tag_ST_CONFIGURATION_HEADER {
    // Number of entries immediately following the header.
    SIZE_T NumEntries;

    // Total byte length: header + entries + string buffer.
    SIZE_T TotalLength;
} ST_CONFIGURATION_HEADER;
