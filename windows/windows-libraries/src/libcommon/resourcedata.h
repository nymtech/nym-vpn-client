#pragma once

#include <cstdint>
#include <windows.h>

namespace common::resourcedata {

struct BinaryResource
{
	size_t size;
	const uint8_t *data;
};

// Load a binary resource from the specified module.
//
// HMODULE/HINSTANCE/Imagebase is the same thing.
//
// The resource doesn't need to be released afterwards.
//
BinaryResource LoadBinaryResource(HMODULE moduleHandle, uint32_t resourceId);

}
