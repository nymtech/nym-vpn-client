#pragma once

#include "iconditionbuilder.h"
#include "filterbuilder.h"
#include "filterengine.h"
#include "providerbuilder.h"
#include "sublayerbuilder.h"

namespace wfp
{

class ObjectInstaller
{
public:

	static bool AddFilter(FilterEngine &engine, FilterBuilder &filterBuilder, const IConditionBuilder &conditionBuilder, UINT64 *id = nullptr);
	static bool AddProvider(FilterEngine &engine, ProviderBuilder &providerBuilder, GUID *key = nullptr);
	static bool AddSublayer(FilterEngine &engine, SublayerBuilder &sublayerBuilder, GUID *key = nullptr);
};

}
