// Copyright 2016-2026 Mullvad VPN AB. All Rights Reserved.
// Copyright 2026 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

#pragma once

//
// All possible states in the driver.
//

enum ST_DRIVER_STATE {
    // Default state after being loaded.
    ST_DRIVER_STATE_NONE = 0,

    // DriverEntry has completed successfully.
    // Basically only driver and device objects are created at this point.
    ST_DRIVER_STATE_STARTED = 1,

    // All subsystems are initialized.
    ST_DRIVER_STATE_INITIALIZED = 2,

    // User mode has registered all processes in the system.
    ST_DRIVER_STATE_READY = 3,

    // IP addresses are registered.
    // A valid configuration is registered.
    ST_DRIVER_STATE_ENGAGED = 4,

    // Driver could not tear down subsystems.
    ST_DRIVER_STATE_ZOMBIE = 5,
};
