#pragma once

#include <guiddef.h>
#include <fwpmu.h>
#include <string>

namespace wfp::conditions {

struct IFilterCondition
{
	virtual std::wstring toString() const = 0;
	virtual const GUID &identifier() const = 0;
	virtual const FWPM_FILTER_CONDITION0 &condition() const = 0;

	virtual ~IFilterCondition() {}
};

}
