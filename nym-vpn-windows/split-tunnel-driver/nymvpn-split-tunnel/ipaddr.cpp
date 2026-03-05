// Copyright 2016-2026 Mullvad VPN AB. All Rights Reserved.
// Copyright 2026 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

#include <wdm.h>
#include "ipaddr.h"
#include "util.h"

namespace ip {

bool ValidTunnelIpv4Address(const ST_IP_ADDRESSES* IpAddresses) {
    return !util::IsEmptyRange(&IpAddresses->TunnelIpv4, sizeof(IpAddresses->TunnelIpv4));
}

bool ValidInternetIpv4Address(const ST_IP_ADDRESSES* IpAddresses) {
    return !util::IsEmptyRange(&IpAddresses->InternetIpv4, sizeof(IpAddresses->InternetIpv4));
}

bool ValidTunnelIpv6Address(const ST_IP_ADDRESSES* IpAddresses) {
    return !util::IsEmptyRange(&IpAddresses->TunnelIpv6, sizeof(IpAddresses->TunnelIpv6));
}

bool ValidInternetIpv6Address(const ST_IP_ADDRESSES* IpAddresses) {
    return !util::IsEmptyRange(&IpAddresses->InternetIpv6, sizeof(IpAddresses->InternetIpv6));
}

} // namespace ip
