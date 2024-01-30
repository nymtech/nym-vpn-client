#include "stdafx.h"
#include "objectdeleter.h"
#include <libcommon/error.h>

namespace wfp
{

//static
void ObjectDeleter::DeleteFilter(FilterEngine &engine, const GUID &key)
{
	auto status = FwpmFilterDeleteByKey0(
		engine.session(),
		&key
	);

	if (ERROR_SUCCESS == status || FWP_E_FILTER_NOT_FOUND == status)
	{
		return;
	}

	THROW_WINDOWS_ERROR(status, "Delete filter")
}

//static
void ObjectDeleter::DeleteFilter(FilterEngine &engine, UINT64 key)
{
	auto status = FwpmFilterDeleteById0(
		engine.session(),
		key
	);

	if (ERROR_SUCCESS == status || FWP_E_FILTER_NOT_FOUND == status)
	{
		return;
	}

	THROW_WINDOWS_ERROR(status, "Delete filter");
}

//static
void ObjectDeleter::DeleteProvider(FilterEngine &engine, const GUID &key)
{
	auto status = FwpmProviderDeleteByKey0(
		engine.session(),
		&key
	);

	if (ERROR_SUCCESS == status || FWP_E_PROVIDER_NOT_FOUND == status)
	{
		return;
	}

	THROW_WINDOWS_ERROR(status, "Delete provider");
}

//static
void ObjectDeleter::DeleteSublayer(FilterEngine &engine, const GUID &key)
{
	auto status = FwpmSubLayerDeleteByKey0(
		engine.session(),
		&key
	);

	if (ERROR_SUCCESS == status || FWP_E_SUBLAYER_NOT_FOUND == status)
	{
		return;
	}

	THROW_WINDOWS_ERROR(status, "Delete sublayer");
}

}
