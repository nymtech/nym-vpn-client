// Copyright 2016-2026 Mullvad VPN AB. All Rights Reserved.
// Copyright 2026 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

#pragma once

//
// Magical include order with defines etc.
// Infuriating.
//

#include <ntddk.h>
#include <wdm.h>
#include <initguid.h>
#pragma warning(push)
#pragma warning(disable : 4201)
#define NDIS630
#include <ndis.h>
#include <fwpsk.h>
#pragma warning(pop)
#include <fwpmk.h>
#include <mstcpip.h>
