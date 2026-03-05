// Copyright 2016-2026 Mullvad VPN AB. All Rights Reserved.
// Copyright 2026 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

#pragma once

#ifdef NTDDI_VERSION // kernel
#ifndef _WIN64
#error Only 64-bit is supported
#endif
#else // user
#ifndef _WIN64
#error Only 64-bit is supported
#endif
#endif
