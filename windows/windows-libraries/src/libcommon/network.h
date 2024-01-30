#pragma once

#include <cstdint>

namespace common::network {

uint32_t MaskFromRoutingPrefix(uint8_t prefix);

//
// Convert array of bytes e.g. 0x7F, 0x0, 0x0, 0x1 into
// uint32_t with network byte order
//
uint32_t LiteralAddressToNetwork(const uint8_t *address);

}
