#pragma once

#include <windows.h>
#include <fwptypes.h>
#include <initializer_list>
#include <string>

namespace wfp
{

//
// Abstraction over a specific IPv4/IPv6 address.
// For IPv4, host byte order is used.
//
class IpAddress
{
public:

	struct Literal;
	struct Literal6;

	// Generic
	IpAddress(const std::wstring &netnode);

	// IPv4
	IpAddress(uint32_t netnode);
	IpAddress(const Literal &netnode);

	// IPv6
	IpAddress(const FWP_BYTE_ARRAY16 &netnode);
	IpAddress(const Literal6 &netnode);

	//
	// Helper-types to construct from literal address
	//

	struct Literal
	{
		Literal(const std::initializer_list<uint8_t> &netnode);
		uint32_t address;
	};

	struct Literal6
	{
		Literal6(const std::initializer_list<uint16_t> &netnode);
		uint8_t address[16];
	};

	enum Type
	{
		Ipv4,
		Ipv6
	};

	Type type() const
	{
		return m_type;
	}

	uint32_t addr() const
	{
		return m_ipv4;
	}

	const FWP_BYTE_ARRAY16 &addr6() const
	{
		return *reinterpret_cast<const FWP_BYTE_ARRAY16 *>(m_ipv6);
	}

	std::wstring toString() const;

	bool operator==(const IpAddress &other) const
	{
		if (Type::Ipv4 == m_type)
		{
			return Type::Ipv4 == other.m_type && m_ipv4 == other.m_ipv4;
		}
		return Type::Ipv6 == other.m_type
			&& 0 == memcmp(m_ipv6, other.m_ipv6, sizeof(m_ipv6));
	}

private:

	uint32_t m_ipv4;
	uint8_t m_ipv6[16];

	Type m_type;
};

}
