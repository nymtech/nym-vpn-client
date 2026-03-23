// Copyright 2016-2026 Mullvad VPN AB. All Rights Reserved.
// Copyright 2026 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

#pragma once

#include <wdm.h>
#include "../defs/events.h"
#include "../defs/types.h"
#include "eventing.h"

namespace eventing {

RAW_EVENT*
BuildStartSplittingEvent(HANDLE ProcessId, ST_SPLITTING_STATUS_CHANGE_REASON Reason, LOWER_UNICODE_STRING* ImageName);

RAW_EVENT*
BuildStopSplittingEvent(HANDLE ProcessId, ST_SPLITTING_STATUS_CHANGE_REASON Reason, LOWER_UNICODE_STRING* ImageName);

RAW_EVENT* BuildStartSplittingErrorEvent(HANDLE ProcessId, LOWER_UNICODE_STRING* ImageName);

RAW_EVENT* BuildStopSplittingErrorEvent(HANDLE ProcessId, LOWER_UNICODE_STRING* ImageName);

RAW_EVENT* BuildErrorMessageEvent(NTSTATUS Status, const UNICODE_STRING* ErrorMessage);

void ReleaseEvent(RAW_EVENT** Event);

} // namespace eventing
