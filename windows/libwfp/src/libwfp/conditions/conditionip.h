#pragma once

#include "comparison.h"
#include "ifiltercondition.h"
#include <libwfp/ipaddress.h>
#include <libwfp/ipnetwork.h>
#include <libcommon/buffer.h>
#include <memory>

namespace wfp::conditions {

class ConditionIp : public IFilterCondition
{
	enum class Type
	{
		NetNode,
		Network,
	};

	ConditionIp(bool local, const IpAddress &netnode, const IStrictComparison &comparison);
	ConditionIp(bool local, const IpNetwork &network, const IStrictComparison &comparison);

public:

	static std::unique_ptr<ConditionIp> Local(const IpAddress &netnode, const IStrictComparison &comparison = CompareEq());
	static std::unique_ptr<ConditionIp> Local(const IpNetwork &network, const IStrictComparison &comparison = CompareEq());
	static std::unique_ptr<ConditionIp> Remote(const IpAddress &netnode, const IStrictComparison &comparison = CompareEq());
	static std::unique_ptr<ConditionIp> Remote(const IpNetwork &network, const IStrictComparison &comparison = CompareEq());

	std::wstring toString() const override;
	const GUID &identifier() const override;
	const FWPM_FILTER_CONDITION0 &condition() const override;

private:

	Type m_type;

	bool m_local;

	const std::shared_ptr<IpAddress> m_netnode;
	const std::shared_ptr<IpNetwork> m_network;

	ComparisonSpecification m_comparison;
	common::Buffer m_assembled;
};

}
