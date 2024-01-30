#include "stdafx.h"
#include "guid.h"
#include "error.h"
#include <rpc.h>

namespace common
{

// static
GUID Guid::Generate()
{
	UUID u;

	const auto status = UuidCreate(&u);

	if (RPC_S_OK != status)
	{
		THROW_WINDOWS_ERROR(status, "Generate GUID");
	}

	return u;
}

// static
GUID Guid::GenerateQuick()
{
	UUID u;

	const auto status = UuidCreateSequential(&u);

	if (RPC_S_OK != status)
	{
		THROW_WINDOWS_ERROR(status, "Generate GUID");
	}

	return u;
}

// static
bool Guid::Empty(const GUID &candidate)
{
	auto data = reinterpret_cast<const uint64_t *>(&candidate);
	return 0 == (*data | *(data + 1));
}

//static
GUID Guid::FromString(const std::wstring &guid)
{
	//
	// There are two standard ways of formatting a GUID
	// One includes embracing curly braces, the other does not
	//

	std::wstring formattedGuid;

	switch (guid.size())
	{
		case 38:
		{
			formattedGuid = guid.substr(1, 36);
			break;
		}
		case 36:
		{
			formattedGuid = guid;
			break;
		}
		default:
		{
			THROW_ERROR("Invalid GUID format");
		}
	}

	GUID convertedGuid;

	auto lolwindows = reinterpret_cast<unsigned short *>(const_cast<wchar_t *>(formattedGuid.c_str()));

	const auto status = UuidFromStringW(lolwindows, &convertedGuid);

	if (RPC_S_OK != status)
	{
		THROW_WINDOWS_ERROR(status, "Convert formatted GUID to raw representation");
	}

	return convertedGuid;
}

}
