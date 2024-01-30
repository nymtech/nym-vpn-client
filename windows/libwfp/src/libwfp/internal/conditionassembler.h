#pragma once

#include <libcommon/buffer.h>
#include <guiddef.h>
#include <fwpmu.h>

namespace wfp::internal {

class ConditionAssembler
{
public:

	static common::Buffer ByteBlob(const GUID &identifier, FWP_MATCH_TYPE matchType, const FWP_BYTE_BLOB &data);
	static common::Buffer Uint16(const GUID &identifier, FWP_MATCH_TYPE matchType, UINT16 value);
	static common::Buffer Uint8(const GUID &identifier, FWP_MATCH_TYPE matchType, UINT8 value);
	static common::Buffer Uint32(const GUID &identifier, FWP_MATCH_TYPE matchType, UINT32 value);
	static common::Buffer ByteArray16(const GUID &identifier, FWP_MATCH_TYPE matchType, const FWP_BYTE_ARRAY16 &array);
	static common::Buffer Range(const GUID &identifier, FWP_MATCH_TYPE matchType, const FWP_RANGE0 &range);
	static common::Buffer Uint64(const GUID &identifier, FWP_MATCH_TYPE matchType, UINT64 value);
	static common::Buffer Ipv4Network(const GUID &identifier, FWP_MATCH_TYPE matchType, const FWP_V4_ADDR_AND_MASK &network);
	static common::Buffer Ipv6Network(const GUID &identifier, FWP_MATCH_TYPE matchType, const FWP_V6_ADDR_AND_MASK &network);
};

}
