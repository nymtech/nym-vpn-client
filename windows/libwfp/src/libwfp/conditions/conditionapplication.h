#pragma once

#include "comparison.h"
#include "ifiltercondition.h"
#include <libcommon/buffer.h>

namespace wfp::conditions {

class ConditionApplication : public IFilterCondition
{
public:

	ConditionApplication(const std::wstring &application, const IStrictComparison &comparison = CompareEq());

	std::wstring toString() const override;
	const GUID &identifier() const override;
	const FWPM_FILTER_CONDITION0 &condition() const override;

private:

	std::wstring m_application;
	ComparisonSpecification m_comparison;

	common::Buffer m_assembled;
};

}
