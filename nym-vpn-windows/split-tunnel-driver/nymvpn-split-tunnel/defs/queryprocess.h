// Copyright 2016-2026 Mullvad VPN AB. All Rights Reserved.
// Copyright 2026 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

#pragma once

//
// Structures related to querying process information.
//

typedef struct tag_ST_QUERY_PROCESS {
    HANDLE ProcessId;
} ST_QUERY_PROCESS;

typedef struct tag_ST_QUERY_PROCESS_RESPONSE {
    HANDLE ProcessId;
    HANDLE ParentProcessId;

    BOOLEAN Split;

    // Byte length for non-null terminated wide char string.
    USHORT ImageNameLength;

    WCHAR ImageName[ANYSIZE_ARRAY];
} ST_QUERY_PROCESS_RESPONSE;
