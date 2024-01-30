#pragma once

#include "filterengine.h"
#include <guiddef.h>
#include <functional>

namespace wfp
{

class ObjectDeleter
{
public:

	static void DeleteFilter(FilterEngine &engine, const GUID &key);
	static void DeleteFilter(FilterEngine &engine, UINT64 key);
	static void DeleteProvider(FilterEngine &engine, const GUID &key);
	static void DeleteSublayer(FilterEngine &engine, const GUID &key);
};

}
