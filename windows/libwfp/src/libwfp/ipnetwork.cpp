#include "stdafx.h"
#include "ipnetwork.h"
#include <libcommon/string.h>
#include <libcommon/network.h>
#include <libcommon/error.h>

namespace wfp
{

IpNetwork::IpNetwork(const IpAddress &addr, uint8_t prefix)
	: m_prefix(prefix)
{
	switch (addr.type())
	{
		case IpAddress::Type::Ipv4:
		{
			m_type = Type::Ipv4;

			m_net.addr = addr.addr();
			m_net.mask = common::network::MaskFromRoutingPrefix(prefix);

			break;
		}
		case IpAddress::Type::Ipv6:
		{
			m_type = Type::Ipv6;

			memcpy(m_net6.addr, addr.addr6().byteArray16, sizeof(m_net6.addr));
			m_net6.prefixLength = prefix;

			break;
		}
		default:
		{
			THROW_ERROR("Missing case handler in switch clause");
		}
	}
}

std::wstring IpNetwork::toString() const
{
	switch (m_type)
	{
		case Type::Ipv4:
		{
			return common::string::FormatIpv4(m_net.addr, m_prefix);
		}
		case Type::Ipv6:
		{
			return common::string::FormatIpv6(m_net6.addr, m_net6.prefixLength);
		}
		default:
		{
			THROW_ERROR("Missing case handler in switch clause");
		}
	}
}

bool IpNetwork::includes(const IpAddress &address) const
{
	if (Type::Ipv4 == m_type)
	{
		if (IpAddress::Type::Ipv4 != address.type())
		{
			return false;
		}

		const auto mask = common::network::MaskFromRoutingPrefix(m_prefix);
		return (address.addr() & mask) == (m_net.addr & mask);
	}

	if (IpAddress::Type::Ipv6 != address.type())
	{
		return false;
	}

	uint32_t numWholeBytes = m_prefix / 8;
	const auto otherArray = address.addr6().byteArray16;

	if (0 != memcmp(m_net6.addr, otherArray, numWholeBytes))
	{
		return false;
	}

	uint32_t byteMask = ~((uint32_t(1) << (8 - (m_prefix % 8))) - 1);
	return (otherArray[numWholeBytes] & byteMask) == (m_net6.addr[numWholeBytes] & byteMask);
}

}
