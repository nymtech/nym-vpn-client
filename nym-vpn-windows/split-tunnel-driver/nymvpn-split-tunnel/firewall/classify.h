// Copyright 2016-2026 Mullvad VPN AB. All Rights Reserved.
// Copyright 2026 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

#pragma once

#include "wfp.h"

namespace firewall {

void ClassificationReset(FWPS_CLASSIFY_OUT0* ClassifyOut);

void ClassificationApplyHardPermit(FWPS_CLASSIFY_OUT0* ClassifyOut);

void ClassificationApplySoftPermit(FWPS_CLASSIFY_OUT0* ClassifyOut);

void ClassificationApplyHardBlock(FWPS_CLASSIFY_OUT0* ClassifyOut);

} // namespace firewall
