#pragma once

#include <windows.h>
#include <fwptypes.h>
#include <string>
#include "ipaddress.h"

namespace wfp
{

class IpNetwork
{
public:

	IpNetwork(const IpAddress &addr, uint8_t prefix);

	enum Type
	{
		Ipv4,
		Ipv6
	};

	Type type() const
	{
		return m_type;
	}

	const FWP_V4_ADDR_AND_MASK &net() const
	{
		return m_net;
	}

	const FWP_V6_ADDR_AND_MASK &net6() const
	{
		return m_net6;
	}

	std::wstring toString() const;

	bool includes(const IpAddress &address) const;

private:

	Type m_type;

	// TODO: Resolve having to store this separately for IPv4 nets.
	uint8_t m_prefix;

	FWP_V4_ADDR_AND_MASK m_net;
	FWP_V6_ADDR_AND_MASK m_net6;
};

}
