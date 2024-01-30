#include "stdafx.h"
#include "resourcedata.h"
#include "error.h"

namespace common::resourcedata {

BinaryResource LoadBinaryResource(HMODULE moduleHandle, uint32_t resourceId)
{
	auto resourceInformationBlock = FindResourceW(moduleHandle, MAKEINTRESOURCEW(resourceId), RT_RCDATA);

	if (nullptr == resourceInformationBlock)
	{
		THROW_WINDOWS_ERROR(GetLastError(), "Locate resource information block");
	}

	BinaryResource result;

	result.size = SizeofResource(moduleHandle, resourceInformationBlock);

	auto resourceHandle = LoadResource(moduleHandle, resourceInformationBlock);

	if (nullptr == resourceHandle)
	{
		THROW_WINDOWS_ERROR(GetLastError(), "Load resource data");
	}

	result.data = reinterpret_cast<uint8_t *>(LockResource(resourceHandle));

	if (nullptr == result.data)
	{
		THROW_ERROR("Failed to lock resource");
	}

	return result;
}

}
