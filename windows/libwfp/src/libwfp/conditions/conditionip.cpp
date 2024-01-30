#include "stdafx.h"
#include "conditionip.h"
#include <libwfp/internal/conditionassembler.h>
#include <libcommon/network.h>
#include <libcommon/string.h>
#include <libcommon/error.h>
#include <sstream>

using ConditionAssembler = ::wfp::internal::ConditionAssembler;

namespace wfp::conditions {

ConditionIp::ConditionIp(bool local, const IpAddress &netnode, const IStrictComparison &comparison)
	: m_type(Type::NetNode)
	, m_local(local)
	, m_netnode(std::make_unique<IpAddress>(netnode))
	, m_comparison(comparison)
{
	switch (netnode.type())
	{
		case IpAddress::Type::Ipv4:
		{
			m_assembled = ConditionAssembler::Uint32(identifier(), m_comparison.op(), netnode.addr());
			break;
		}
		case IpAddress::Type::Ipv6:
		{
			m_assembled = ConditionAssembler::ByteArray16(identifier(), m_comparison.op(), netnode.addr6());
			break;
		}
		default:
		{
			THROW_ERROR("Missing case handler in switch clause");
		}
	}
}

ConditionIp::ConditionIp(bool local, const IpNetwork &network, const IStrictComparison &comparison)
	: m_type(Type::Network)
	, m_local(local)
	, m_network(std::make_unique<IpNetwork>(network))
	, m_comparison(comparison)
{
	switch (network.type())
	{
		case IpNetwork::Type::Ipv4:
		{
			m_assembled = ConditionAssembler::Ipv4Network(identifier(), m_comparison.op(), network.net());
			break;
		}
		case IpAddress::Type::Ipv6:
		{
			m_assembled = ConditionAssembler::Ipv6Network(identifier(), m_comparison.op(), network.net6());
			break;
		}
		default:
		{
			THROW_ERROR("Missing case handler in switch clause");
		}
	}
}

//static
std::unique_ptr<ConditionIp> ConditionIp::Local(const IpAddress &netnode, const IStrictComparison &comparison)
{
	return std::unique_ptr<ConditionIp>(new ConditionIp(true, netnode, comparison));
}

//static
std::unique_ptr<ConditionIp> ConditionIp::Local(const IpNetwork &network, const IStrictComparison &comparison)
{
	return std::unique_ptr<ConditionIp>(new ConditionIp(true, network, comparison));
}

//static
std::unique_ptr<ConditionIp> ConditionIp::Remote(const IpAddress &netnode, const IStrictComparison &comparison)
{
	return std::unique_ptr<ConditionIp>(new ConditionIp(false, netnode, comparison));
}

//static
std::unique_ptr<ConditionIp> ConditionIp::Remote(const IpNetwork &network, const IStrictComparison &comparison)
{
	return std::unique_ptr<ConditionIp>(new ConditionIp(false, network, comparison));
}

std::wstring ConditionIp::toString() const
{
	std::wstringstream ss;

	const auto target = (m_local ? L"local" : L"remote");

	switch (m_type)
	{
		case Type::NetNode:
		{
			ss << target << L" ip " << m_comparison.toString() << L" " << m_netnode->toString();
			break;
		}
		case Type::Network:
		{
			ss << target << L" ip net " << m_comparison.toString() << L" " << m_network->toString();
			break;
		}
		default:
		{
			THROW_ERROR("Missing case handler in switch clause");
		}
	};

	return ss.str();
}

const GUID &ConditionIp::identifier() const
{
	return (m_local ? FWPM_CONDITION_IP_LOCAL_ADDRESS : FWPM_CONDITION_IP_REMOTE_ADDRESS);
}

const FWPM_FILTER_CONDITION0 &ConditionIp::condition() const
{
	return *reinterpret_cast<FWPM_FILTER_CONDITION0 *>(m_assembled.data());
}

}
