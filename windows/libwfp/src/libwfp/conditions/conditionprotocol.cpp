#include "stdafx.h"
#include "conditionprotocol.h"
#include <libwfp/internal/conditionassembler.h>
#include <winsock2.h>
#include <libcommon/error.h>

using ConditionAssembler = ::wfp::internal::ConditionAssembler;

namespace wfp::conditions {

ConditionProtocol::ConditionProtocol(Protocol protocol)
	: m_protocol(protocol)
{
	m_assembled = ConditionAssembler::Uint8
	(
		identifier(),
		FWP_MATCH_EQUAL,
		TranslateProtocol(protocol)
	);
}

std::wstring ConditionProtocol::toString() const
{
	switch (m_protocol)
	{
		case Protocol::Tcp: return L"protocol = TCP";
		case Protocol::Udp: return L"protocol = UDP";
		case Protocol::Icmp: return L"protocol = ICMP";
		case Protocol::IcmpV6: return L"protocol = ICMPv6";
		case Protocol::Ip: return L"protocol = IP";
		case Protocol::IpV6: return L"protocol = IPv6";
		case Protocol::Raw: return L"protocol = Raw";
		default:
		{
			THROW_ERROR("Missing case handler");
		}
	}
}

const GUID &ConditionProtocol::identifier() const
{
	return FWPM_CONDITION_IP_PROTOCOL;
}

const FWPM_FILTER_CONDITION0 &ConditionProtocol::condition() const
{
	return *reinterpret_cast<FWPM_FILTER_CONDITION0 *>(m_assembled.data());
}

//static
std::unique_ptr<ConditionProtocol> ConditionProtocol::Tcp()
{
	return std::unique_ptr<ConditionProtocol>(new ConditionProtocol(Protocol::Tcp));
}

//static
std::unique_ptr<ConditionProtocol> ConditionProtocol::Udp()
{
	return std::unique_ptr<ConditionProtocol>(new ConditionProtocol(Protocol::Udp));
}

//static
std::unique_ptr<ConditionProtocol> ConditionProtocol::Icmp()
{
	return std::unique_ptr<ConditionProtocol>(new ConditionProtocol(Protocol::Icmp));
}

//static
std::unique_ptr<ConditionProtocol> ConditionProtocol::IcmpV6()
{
	return std::unique_ptr<ConditionProtocol>(new ConditionProtocol(Protocol::IcmpV6));
}

//static
std::unique_ptr<ConditionProtocol> ConditionProtocol::Ip()
{
	return std::unique_ptr<ConditionProtocol>(new ConditionProtocol(Protocol::Ip));
}

//static
std::unique_ptr<ConditionProtocol> ConditionProtocol::IpV6()
{
	return std::unique_ptr<ConditionProtocol>(new ConditionProtocol(Protocol::IpV6));
}

//static
std::unique_ptr<ConditionProtocol> ConditionProtocol::Raw()
{
	return std::unique_ptr<ConditionProtocol>(new ConditionProtocol(Protocol::Raw));
}

// static
uint8_t ConditionProtocol::TranslateProtocol(Protocol protocol)
{
	switch (protocol)
	{
		case Protocol::Tcp: return IPPROTO_TCP;
		case Protocol::Udp: return IPPROTO_UDP;
		case Protocol::Icmp: return IPPROTO_ICMP;
		case Protocol::IcmpV6: return IPPROTO_ICMPV6;
		case Protocol::Ip: return IPPROTO_IPV4;
		case Protocol::IpV6: return IPPROTO_IPV6;
		case Protocol::Raw: return IPPROTO_RAW;
		default:
		{
			THROW_ERROR("Missing case handler");
		}
	}
}

}
