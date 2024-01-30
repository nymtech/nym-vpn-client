#pragma once

#include "comparison.h"
#include "ifiltercondition.h"
#include <libcommon/buffer.h>
#include <cstdint>
#include <memory>

namespace wfp::conditions {

class ConditionIcmp : public IFilterCondition
{
	enum class ConditionType
	{
		Type,
		Code
	};

	ConditionIcmp(ConditionType type, uint16_t value, const IRelaxedComparison &comparison);

public:

	static std::unique_ptr<ConditionIcmp> Type(uint16_t value, const IRelaxedComparison &comparison = CompareEq());
	static std::unique_ptr<ConditionIcmp> Code(uint16_t value, const IRelaxedComparison &comparison = CompareEq());

	std::wstring toString() const override;
	const GUID &identifier() const override;
	const FWPM_FILTER_CONDITION0 &condition() const override;

private:

	ConditionType m_type;
	uint16_t m_value;
	ComparisonSpecification m_comparison;

	common::Buffer m_assembled;
};

}
