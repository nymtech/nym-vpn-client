#pragma once

#include <string>
#include <guiddef.h>

namespace common
{

class Guid
{
public:

	Guid() = delete;

	static GUID Generate();

	// This is slightly faster but may return a GUID that is not globally
	// unique, rather, only unique on the local machine.
	static GUID GenerateQuick();

	static bool Empty(const GUID &candidate);

	static GUID FromString(const std::wstring &guid);
};

}
