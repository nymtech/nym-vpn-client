// Copyright 2016-2026 Mullvad VPN AB. All Rights Reserved.
// Copyright 2026 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

#pragma once

enum ST_EVENT_ID {
    ST_EVENT_ID_START_SPLITTING_PROCESS = 0, // ST_SPLITTING_EVENT
    ST_EVENT_ID_STOP_SPLITTING_PROCESS,      // ST_SPLITTING_EVENT

    ST_EVENT_ID_ERROR_FLAG = 0x80000000,

    ST_EVENT_ID_ERROR_START_SPLITTING_PROCESS, // ST_SPLITTING_ERROR_EVENT
    ST_EVENT_ID_ERROR_STOP_SPLITTING_PROCESS,  // ST_SPLITTING_ERROR_EVENT

    ST_EVENT_ID_ERROR_MESSAGE // ST_ERROR_MESSAGE_EVENT
};

typedef struct tag_ST_EVENT_HEADER {
    ST_EVENT_ID EventId;

    // Size of payload.
    SIZE_T EventSize;

    // Message defined payload.
    UCHAR EventData[ANYSIZE_ARRAY];
} ST_EVENT_HEADER;

enum ST_SPLITTING_STATUS_CHANGE_REASON {
    ST_SPLITTING_REASON_BY_INHERITANCE = 1,
    ST_SPLITTING_REASON_BY_CONFIG = 2,
    ST_SPLITTING_REASON_PROCESS_ARRIVING = 4,
    ST_SPLITTING_REASON_PROCESS_DEPARTING = 8
};

typedef struct tag_ST_SPLITTING_EVENT {
    HANDLE ProcessId;

    ST_SPLITTING_STATUS_CHANGE_REASON Reason;

    // Byte length for non-null terminated wide char string.
    USHORT ImageNameLength;

    WCHAR ImageName[ANYSIZE_ARRAY];
} ST_SPLITTING_EVENT;

typedef struct tag_ST_SPLITTING_ERROR_EVENT {
    HANDLE ProcessId;

    // Byte length for non-null terminated wide char string.
    USHORT ImageNameLength;

    WCHAR ImageName[ANYSIZE_ARRAY];
} ST_SPLITTING_ERROR_EVENT;

typedef struct tag_ST_ERROR_MESSAGE_EVENT {
    NTSTATUS Status;

    // Byte length for non-null terminated wide char string.
    USHORT ErrorMessageLength;

    WCHAR ErrorMessage[ANYSIZE_ARRAY];
} ST_ERROR_MESSAGE_EVENT;
