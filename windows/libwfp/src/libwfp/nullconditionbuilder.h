#pragma once

#include "iconditionbuilder.h"

namespace wfp
{

struct NullConditionBuilder : public IConditionBuilder
{
	bool build(ConditionSink sink) const override
	{
		return sink(nullptr, 0);
	}
};

}
