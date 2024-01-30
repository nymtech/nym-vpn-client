#pragma once

#include <guiddef.h>

namespace wfp
{

struct LayerConditions
{
	static bool IsCompatible(const GUID &layer, const GUID &condition);
};

}
