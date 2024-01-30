#include "stdafx.h"
#include "network.h"
#include <winsock2.h>

namespace common::network {

uint32_t MaskFromRoutingPrefix(uint8_t prefix)
{
	if (0 == prefix)
	{
		return 0;
	}

	return ~((uint32_t(1) << (32 - prefix)) - 1);
}

uint32_t LiteralAddressToNetwork(const uint8_t *address)
{
	return htonl(*reinterpret_cast<const uint32_t *>(address));
}

}
