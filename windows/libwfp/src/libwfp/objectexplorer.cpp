#include <stdafx.h>
#include "objectexplorer.h"

namespace wfp
{

//static
bool ObjectExplorer::GetFilter(FilterEngine &engine, const GUID &key, std::function<bool(const FWPM_FILTER0 &)> callback)
{
	FWPM_FILTER0 *filter;

	auto status = FwpmFilterGetByKey0(
		engine.session(),
		&key,
		&filter
	);

	if (status != ERROR_SUCCESS)
	{
		return false;
	}

	auto result = callback(*filter);

	FwpmFreeMemory0((void**)&filter);

	return result;
}

//static
bool ObjectExplorer::GetFilter(FilterEngine &engine, UINT64 id, std::function<bool(const FWPM_FILTER0 &)> callback)
{
	FWPM_FILTER0 *filter;

	auto status = FwpmFilterGetById0(
		engine.session(),
		id,
		&filter
	);

	if (status != ERROR_SUCCESS)
	{
		return false;
	}

	auto result = callback(*filter);

	FwpmFreeMemory0((void**)&filter);

	return result;
}

//static
bool ObjectExplorer::GetLayer(FilterEngine &engine, const GUID &key, std::function<bool(const FWPM_LAYER0 &)> callback)
{
	FWPM_LAYER0 *layer;

	auto status = FwpmLayerGetByKey0(
		engine.session(),
		&key,
		&layer
	);

	if (status != ERROR_SUCCESS)
	{
		return false;
	}

	auto result = callback(*layer);

	FwpmFreeMemory0((void**)&layer);

	return result;
}

//static
bool ObjectExplorer::GetLayer(FilterEngine &engine, UINT16 id, std::function<bool(const FWPM_LAYER0 &)> callback)
{
	FWPM_LAYER0 *layer;

	auto status = FwpmLayerGetById0(
		engine.session(),
		id,
		&layer
	);

	if (status != ERROR_SUCCESS)
	{
		return false;
	}

	auto result = callback(*layer);

	FwpmFreeMemory0((void**)&layer);

	return result;
}

//static
bool ObjectExplorer::GetProvider(FilterEngine &engine, const GUID &key, std::function<bool(const FWPM_PROVIDER0 &)> callback)
{
	FWPM_PROVIDER0 *provider;

	auto status = FwpmProviderGetByKey0(
		engine.session(),
		&key,
		&provider
	);

	if (status != ERROR_SUCCESS)
	{
		return false;
	}

	auto result = callback(*provider);

	FwpmFreeMemory0((void**)&provider);

	return result;
}

//static
bool ObjectExplorer::GetSublayer(FilterEngine &engine, const GUID &key, std::function<bool(const FWPM_SUBLAYER0 &)> callback)
{
	FWPM_SUBLAYER0 *sublayer;

	auto status = FwpmSubLayerGetByKey0(
		engine.session(),
		&key,
		&sublayer
	);

	if (status != ERROR_SUCCESS)
	{
		return false;
	}

	auto result = callback(*sublayer);

	FwpmFreeMemory0((void**)&sublayer);

	return result;
}

}
