#include "stdafx.h"
#include "ipaddress.h"
#include <libcommon/string.h>
#include <libcommon/error.h>
#include <ws2tcpip.h>
#include <winsock2.h>

namespace
{

uint32_t PackIpv4(uint8_t a, uint8_t b, uint8_t c, uint8_t d)
{
	return ((uint32_t(a) << 24) | (uint32_t(b) << 16) | (uint32_t(c) << 8) | d);
}

} // anonymous namespace

namespace wfp
{

IpAddress::Literal::Literal(const std::initializer_list<uint8_t> &netnode)
{
	if (netnode.size() != 4)
	{
		THROW_ERROR("Invalid IPv4 specification");
	}

	auto it = netnode.begin();

	auto a = *it++;
	auto b = *it++;
	auto c = *it++;
	auto d = *it++;

	address = PackIpv4(a, b, c, d);
}

IpAddress::Literal6::Literal6(const std::initializer_list<uint16_t> &netnode)
{
	if (netnode.size() != 8)
	{
		THROW_ERROR("Invalid IPv6 specification");
	}

	size_t index = 0;

	for (uint16_t hextet : netnode)
	{
		address[index++] = uint8_t((hextet & 0xFF00) >> 8);
		address[index++] = uint8_t(hextet & 0xFF);
	}
}

IpAddress::IpAddress(const std::wstring &netnode)
{
	uint8_t buffer[sizeof(IN6_ADDR)];

	if (1 == InetPtonW(AF_INET, netnode.c_str(), &buffer))
	{
		auto parsed = reinterpret_cast<const IN_ADDR *>(&buffer);

		m_ipv4 = ntohl(parsed->s_addr);
		m_type = Type::Ipv4;

		memset(m_ipv6, 0, sizeof(m_ipv6));
	}
	else if (1 == InetPtonW(AF_INET6, netnode.c_str(), &buffer))
	{
		auto parsed = reinterpret_cast<const IN6_ADDR *>(&buffer);

		memcpy(m_ipv6, parsed->u.Byte, sizeof(m_ipv6));
		m_type = Type::Ipv6;

		m_ipv4 = 0;
	}
	else
	{
		THROW_ERROR("Parser failure: Invalid IP address");
	}
}

IpAddress::IpAddress(uint32_t netnode)
	: m_ipv4(netnode)
	, m_type(Type::Ipv4)
{
	memset(m_ipv6, 0, sizeof(m_ipv6));
}

IpAddress::IpAddress(const IpAddress::Literal &netnode)
	: IpAddress(netnode.address)
{
}

IpAddress::IpAddress(const FWP_BYTE_ARRAY16 &netnode)
	: m_ipv4(0)
	, m_type(Type::Ipv6)
{
	memcpy(m_ipv6, netnode.byteArray16, sizeof(m_ipv6));
}

IpAddress::IpAddress(const IpAddress::Literal6 &netnode)
	: m_ipv4(0)
	, m_type(Type::Ipv6)
{
	memcpy(m_ipv6, netnode.address, sizeof(m_ipv6));
}

std::wstring IpAddress::toString() const
{
	switch (m_type)
	{
		case Type::Ipv4:
		{
			return common::string::FormatIpv4(m_ipv4);
		}
		case Type::Ipv6:
		{
			return common::string::FormatIpv6(m_ipv6);
		}
		default:
		{
			THROW_ERROR("Missing case handler in switch clause");
		}
	}
}

}
