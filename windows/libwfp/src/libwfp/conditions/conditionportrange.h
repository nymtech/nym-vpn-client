#pragma once

#include "comparison.h"
#include "ifiltercondition.h"
#include <libcommon/buffer.h>
#include <memory>

namespace wfp::conditions {

class ConditionPortRange : public IFilterCondition
{
	ConditionPortRange(bool local, uint16_t firstPort, uint16_t lastPort, const IStrictComparison &comparison);

public:

	static std::unique_ptr<ConditionPortRange> Local(uint16_t firstPort, uint16_t lastPort, const IStrictComparison &comparison = CompareEq());
	static std::unique_ptr<ConditionPortRange> Remote(uint16_t firstPort, uint16_t lastPort, const IStrictComparison &comparison = CompareEq());

	std::wstring toString() const override;
	const GUID &identifier() const override;
	const FWPM_FILTER_CONDITION0 &condition() const override;

private:

	bool m_local;

	uint16_t m_firstPort;
	uint16_t m_lastPort;
	ComparisonSpecification m_comparison;

	common::Buffer m_assembled;
};

}
