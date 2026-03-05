// Copyright 2016-2026 Mullvad VPN AB. All Rights Reserved.
// Copyright 2026 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

#pragma once

#include <wdm.h>
#include <wdf.h>
#include "containers/procregistry.h"
#include "containers/registeredimage.h"

//
// The single instance of this struct lives in the device context.
// But it has to be defined here so it can be shared with other components
// in the system that should not be concerned with the full context.
//
struct PROCESS_REGISTRY_MGMT {
    WDFSPINLOCK Lock;
    procregistry::CONTEXT* Instance;
};

//
// Same deal as above.
//
// This instance is replaced from time to time hence wrapping it makes
// for a better interface when sharing it.
//
struct REGISTERED_IMAGE_MGMT {
    registeredimage::CONTEXT* volatile Instance;
};
