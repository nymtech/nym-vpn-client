#pragma once

#include "comparison.h"
#include "ifiltercondition.h"
#include <libcommon/buffer.h>

namespace wfp::conditions {

class ConditionLoopback : public IFilterCondition
{
public:

	enum class Type
	{
		LoopbackInterface,
		LoopbackTraffic
	};

	ConditionLoopback(Type type = Type::LoopbackTraffic, const IStrictComparison &comparison = CompareEq());

	std::wstring toString() const override;
	const GUID &identifier() const override;
	const FWPM_FILTER_CONDITION0 &condition() const override;

private:

	Type m_type;
	ComparisonSpecification m_comparison;
	common::Buffer m_assembled;
};

}
