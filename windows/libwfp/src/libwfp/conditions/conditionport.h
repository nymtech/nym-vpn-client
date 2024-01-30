#pragma once

#include "comparison.h"
#include "ifiltercondition.h"
#include <libcommon/buffer.h>
#include <memory>

namespace wfp::conditions {

class ConditionPort : public IFilterCondition
{
	ConditionPort(bool local, uint16_t port, const IRelaxedComparison &comparison);

public:

	static std::unique_ptr<ConditionPort> Local(uint16_t port, const IRelaxedComparison &comparison = CompareEq());
	static std::unique_ptr<ConditionPort> Remote(uint16_t port, const IRelaxedComparison &comparison = CompareEq());

	std::wstring toString() const override;
	const GUID &identifier() const override;
	const FWPM_FILTER_CONDITION0 &condition() const override;

private:

	bool m_local;

	uint16_t m_port;
	ComparisonSpecification m_comparison;

	common::Buffer m_assembled;
};

}
