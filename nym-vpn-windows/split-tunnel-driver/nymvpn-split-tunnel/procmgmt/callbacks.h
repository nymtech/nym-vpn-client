// Copyright 2016-2026 Mullvad VPN AB. All Rights Reserved.
// Copyright 2026 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

#pragma once

#include <wdf.h>

namespace procmgmt {

typedef void(NTAPI* ACQUIRE_STATE_LOCK_FN)(void* context);
typedef void(NTAPI* RELEASE_STATE_LOCK_FN)(void* context);
typedef bool(NTAPI* ENGAGED_STATE_ACTIVE_FN)(void* context);

} // namespace procmgmt
