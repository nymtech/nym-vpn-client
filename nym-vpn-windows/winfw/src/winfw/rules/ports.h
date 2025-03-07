#pragma once

#include <cstdint>

namespace rules
{

// Use weakly typed enum to get implicit promotion to integral types.
enum Ports : uint16_t
{
	DHCPV4_CLIENT_PORT = 68,
	DHCPV4_SERVER_PORT = 67,
	DHCPV6_CLIENT_PORT = 546,
	DHCPV6_SERVER_PORT = 547,

	DNS_SERVER_PORT = 53,
	DNS_OVER_HTTPS_PORT = 443,
	DNS_OVER_TLS_PORT = 853,
};


static const uint16_t DNS_PORTS[] = {
	DNS_SERVER_PORT,
	DNS_OVER_HTTPS_PORT,
	DNS_OVER_TLS_PORT
};

}
