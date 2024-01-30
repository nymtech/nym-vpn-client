#include "stdafx.h"
#include "conditionbuilder.h"
#include "layerconditions.h"
#include <libcommon/error.h>

using IFilterCondition = wfp::conditions::IFilterCondition;

namespace wfp
{

ConditionBuilder::ConditionBuilder(const GUID &layer)
{
	memcpy(&m_layer, &layer, sizeof(layer));
}

ConditionBuilder &ConditionBuilder::add_condition(std::unique_ptr<IFilterCondition> condition)
{
	if (false == LayerConditions::IsCompatible(m_layer, condition->identifier()))
	{
		THROW_ERROR("Condition is not compatible with layer");
	}

	m_conditions.insert(std::move(condition));

	return *this;
}

bool ConditionBuilder::build(ConditionSink sink) const
{
	auto conditions = std::make_unique<FWPM_FILTER_CONDITION0[]>(m_conditions.size());

	//
	// Individual conditions must be kept around for the lifetime of 'conditions'
	// This is because individual conditions manage data pointed by entries in 'conditions'
	//

	auto index = 0;

	for (const std::unique_ptr<IFilterCondition> &condition : m_conditions)
	{
		memcpy(&conditions[index++], &condition->condition(), sizeof(FWPM_FILTER_CONDITION0));
	}

	return sink(conditions.get(), m_conditions.size());
}

}
