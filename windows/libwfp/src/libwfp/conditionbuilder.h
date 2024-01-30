#pragma once

#include "iconditionbuilder.h"
#include "conditions/ifiltercondition.h"
#include <guiddef.h>
#include <cstring>
#include <memory>
#include <set>

namespace wfp
{

typedef std::function<bool(FWPM_FILTER_CONDITION0 *, size_t)> ConditionSink;

class ConditionBuilder : public IConditionBuilder
{
public:

	ConditionBuilder(const GUID &layer);

	ConditionBuilder &add_condition(std::unique_ptr<conditions::IFilterCondition> condition);

	bool build(ConditionSink sink) const override;

	void reset()
	{
		m_conditions.clear();
	}

	void reset(const GUID &layer)
	{
		memcpy(&m_layer, &layer, sizeof(layer));
		reset();
	}

private:

	GUID m_layer;

	struct FilterConditionLess
	{
		bool operator()(const std::unique_ptr<conditions::IFilterCondition> &lhs, const std::unique_ptr<conditions::IFilterCondition> &rhs) const
		{
			return memcmp(&(lhs->identifier()), &(rhs->identifier()), sizeof(GUID)) < 0;
		}
	};

	//
	// This container provides automatic sorting of conditions
	//
	// This is required in order for BFE to accept duplicate conditions
	// and evaluate them using the OR operator
	//
	std::multiset<std::unique_ptr<conditions::IFilterCondition>, FilterConditionLess> m_conditions;
};

}
