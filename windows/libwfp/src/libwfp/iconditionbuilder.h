#pragma once

#include <fwpmu.h>
#include <functional>

namespace wfp
{

typedef std::function<bool(FWPM_FILTER_CONDITION0 *, size_t)> ConditionSink;

struct IConditionBuilder
{
	virtual ~IConditionBuilder() = 0
	{
	}

	virtual bool build(ConditionSink sink) const = 0;
};

}
