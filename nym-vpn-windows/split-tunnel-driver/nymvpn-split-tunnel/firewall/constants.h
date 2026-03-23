// Copyright 2016-2026 Mullvad VPN AB. All Rights Reserved.
// Copyright 2026 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

#pragma once

#include <wdm.h>

namespace {

static const UINT64 ST_MAX_FILTER_WEIGHT = MAXUINT64;
static const UINT64 ST_HIGH_FILTER_WEIGHT = MAXUINT64 - 10;

static const UINT16 DNS_SERVER_PORT = 53;

} // anonymous namespace
