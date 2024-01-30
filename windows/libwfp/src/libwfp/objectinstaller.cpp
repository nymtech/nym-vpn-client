#include "stdafx.h"
#include "objectinstaller.h"
#include <libcommon/error.h>
#include <libcommon/guid.h>
#include <guiddef.h>
#include <fwpmu.h>

namespace wfp
{

//static
bool ObjectInstaller::AddFilter(FilterEngine &engine, FilterBuilder &filterBuilder, const IConditionBuilder &conditionBuilder, UINT64 *id)
{
	return conditionBuilder.build([&](FWPM_FILTER_CONDITION0 *conditions, size_t numConditions)
	{
		return filterBuilder.build([&](FWPM_FILTER0 &filter)
		{
			filter.filterCondition = conditions;
			filter.numFilterConditions = static_cast<UINT32>(numConditions);

			auto status = FwpmFilterAdd0(
				engine.session(),
				&filter,
				nullptr,
				id
			);

			if (ERROR_SUCCESS != status)
			{
				THROW_WINDOWS_ERROR(status, "Register filter with BFE");
			}

			return true;
		});
	});
}

//static
bool ObjectInstaller::AddProvider(FilterEngine &engine, ProviderBuilder &providerBuilder, GUID *key)
{
	return providerBuilder.build([&](FWPM_PROVIDER0 &provider)
	{
		if (nullptr != key && common::Guid::Empty(provider.providerKey))
		{
			provider.providerKey = common::Guid::GenerateQuick();
		}

		auto status = FwpmProviderAdd0(
			engine.session(),
			&provider,
			nullptr
		);

		if (ERROR_SUCCESS != status)
		{
			THROW_WINDOWS_ERROR(status, "Register provider with BFE");
		}

		if (nullptr != key)
		{
			*key = provider.providerKey;
		}

		return true;
	});
}

//static
bool ObjectInstaller::AddSublayer(FilterEngine &engine, SublayerBuilder &sublayerBuilder, GUID *key)
{
	return sublayerBuilder.build([&](FWPM_SUBLAYER0 &sublayer)
	{
		if (nullptr != key && common::Guid::Empty(sublayer.subLayerKey))
		{
			sublayer.subLayerKey = common::Guid::GenerateQuick();
		}

		auto status = FwpmSubLayerAdd0(
			engine.session(),
			&sublayer,
			nullptr
		);

		if (ERROR_SUCCESS != status)
		{
			THROW_WINDOWS_ERROR(status, "Register sublayer with BFE");
		}

		if (nullptr != key)
		{
			*key = sublayer.subLayerKey;
		}

		return true;
	});
}

}
