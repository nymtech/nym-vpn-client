#pragma once

#include <cassert>

namespace common::math {

inline size_t RoundPowerTwo(size_t value, size_t r)
{
	// power of two
	assert(r > 1 && (r & (r - 1)) == 0);

	return ((value + r - 1) & ~(r - 1));
}

}
