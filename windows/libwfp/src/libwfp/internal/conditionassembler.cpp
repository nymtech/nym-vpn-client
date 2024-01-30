#include "stdafx.h"
#include "conditionassembler.h"
#include <libcommon/binarycomposer.h>

using Buffer = ::common::Buffer;
using ConstBufferView = ::common::ConstBufferView;

#define MAKE_PTR(ptrbase, ptroffset, datatype) reinterpret_cast<datatype *>(reinterpret_cast<uint8_t *>(ptrbase) + ptroffset)

namespace wfp::internal {

//static
Buffer ConditionAssembler::ByteBlob(
	const GUID &identifier,
	FWP_MATCH_TYPE matchType,
	const FWP_BYTE_BLOB &blob)
{
	FWPM_FILTER_CONDITION0 definition;

	memcpy(&definition.fieldKey, &identifier, sizeof(identifier));
	definition.matchType = matchType;
	definition.conditionValue.type = FWP_BYTE_BLOB_TYPE;

	//
	// Goal, buffer with:
	//
	// FWPM_FILTER_CONDITION0
	// FWP_BYTE_BLOB
	// raw blob data
	//

	auto composer = ::common::BinaryComposer
	{
		ConstBufferView(&definition, sizeof(definition)),
		ConstBufferView(&blob, sizeof(blob)),
		ConstBufferView(blob.data, blob.size)
	};

	auto buffer = composer.buffer();
	auto offsets = composer.offsets();

	//
	// Correct stored pointers
	//

	auto finalDefinition = MAKE_PTR(buffer, offsets[0], FWPM_FILTER_CONDITION0);
	auto finalBlob = MAKE_PTR(buffer, offsets[1], FWP_BYTE_BLOB);
	auto finalData = MAKE_PTR(buffer, offsets[2], uint8_t);

	finalDefinition->conditionValue.byteBlob = finalBlob;
	finalBlob->data = finalData;

	return composer.acquire();
}

//static
Buffer ConditionAssembler::Uint16(
	const GUID &identifier,
	FWP_MATCH_TYPE matchType,
	UINT16 value)
{
	auto definition = std::make_unique<FWPM_FILTER_CONDITION0>();

	memcpy(&definition->fieldKey, &identifier, sizeof(identifier));
	definition->matchType = matchType;
	definition->conditionValue.type = FWP_UINT16;
	definition->conditionValue.uint16 = value;

	return Buffer(definition.release(), sizeof(FWPM_FILTER_CONDITION0));
}

//static
Buffer ConditionAssembler::Uint8(
	const GUID &identifier,
	FWP_MATCH_TYPE matchType,
	UINT8 value)
{
	auto definition = std::make_unique<FWPM_FILTER_CONDITION0>();

	memcpy(&definition->fieldKey, &identifier, sizeof(identifier));
	definition->matchType = matchType;
	definition->conditionValue.type = FWP_UINT8;
	definition->conditionValue.uint8 = value;

	return Buffer(definition.release(), sizeof(FWPM_FILTER_CONDITION0));
}

//static
Buffer ConditionAssembler::Uint32(
	const GUID &identifier,
	FWP_MATCH_TYPE matchType,
	UINT32 value)
{
	auto definition = std::make_unique<FWPM_FILTER_CONDITION0>();

	memcpy(&definition->fieldKey, &identifier, sizeof(identifier));
	definition->matchType = matchType;
	definition->conditionValue.type = FWP_UINT32;
	definition->conditionValue.uint32 = value;

	return Buffer(definition.release(), sizeof(FWPM_FILTER_CONDITION0));
}

//static
Buffer ConditionAssembler::ByteArray16(
	const GUID &identifier,
	FWP_MATCH_TYPE matchType,
	const FWP_BYTE_ARRAY16 &array)
{
	FWPM_FILTER_CONDITION0 definition;

	memcpy(&definition.fieldKey, &identifier, sizeof(identifier));
	definition.matchType = matchType;
	definition.conditionValue.type = FWP_BYTE_ARRAY16_TYPE;

	//
	// Goal, buffer with:
	//
	// FWPM_FILTER_CONDITION0
	// FWP_BYTE_ARRAY16
	//

	auto composer = ::common::BinaryComposer
	{
		ConstBufferView(&definition, sizeof(definition)),
		ConstBufferView(&array, sizeof(array))
	};

	auto buffer = composer.buffer();
	auto offsets = composer.offsets();

	//
	// Correct stored pointer
	//

	auto finalDefinition = MAKE_PTR(buffer, offsets[0], FWPM_FILTER_CONDITION0);
	auto finalArray = MAKE_PTR(buffer, offsets[1], FWP_BYTE_ARRAY16);

	finalDefinition->conditionValue.byteArray16 = finalArray;

	return composer.acquire();
}

//static
Buffer ConditionAssembler::Range(
	const GUID &identifier,
	FWP_MATCH_TYPE matchType,
	const FWP_RANGE0 &range)
{
	FWPM_FILTER_CONDITION0 definition;

	memcpy(&definition.fieldKey, &identifier, sizeof(identifier));
	definition.matchType = matchType;
	definition.conditionValue.type = FWP_RANGE_TYPE;

	//
	// Goal, buffer with:
	//
	// FWPM_FILTER_CONDITION0
	// FWP_RANGE0
	//

	auto composer = ::common::BinaryComposer
	{
		ConstBufferView(&definition, sizeof(definition)),
		ConstBufferView(&range, sizeof(range))
	};

	auto buffer = composer.buffer();
	auto offsets = composer.offsets();

	//
	// Correct stored pointer
	//

	auto finalDefinition = MAKE_PTR(buffer, offsets[0], FWPM_FILTER_CONDITION0);
	auto finalRange = MAKE_PTR(buffer, offsets[1], FWP_RANGE0);

	finalDefinition->conditionValue.rangeValue = finalRange;

	return composer.acquire();
}

//static
Buffer ConditionAssembler::Uint64(
	const GUID &identifier,
	FWP_MATCH_TYPE matchType,
	UINT64 value)
{
	FWPM_FILTER_CONDITION0 definition;

	memcpy(&definition.fieldKey, &identifier, sizeof(identifier));
	definition.matchType = matchType;
	definition.conditionValue.type = FWP_UINT64;

	//
	// Goal, buffer with:
	//
	// FWPM_FILTER_CONDITION0
	// UINT64
	//

	auto composer = ::common::BinaryComposer
	{
		ConstBufferView(&definition, sizeof(definition)),
		ConstBufferView(&value, sizeof(value))
	};

	auto buffer = composer.buffer();
	auto offsets = composer.offsets();

	//
	// Correct stored pointer
	//

	auto finalDefinition = MAKE_PTR(buffer, offsets[0], FWPM_FILTER_CONDITION0);
	auto finalValue = MAKE_PTR(buffer, offsets[1], UINT64);

	finalDefinition->conditionValue.uint64 = finalValue;

	return composer.acquire();
}

//static
Buffer ConditionAssembler::Ipv4Network(
	const GUID &identifier,
	FWP_MATCH_TYPE matchType,
	const FWP_V4_ADDR_AND_MASK &network)
{
	FWPM_FILTER_CONDITION0 definition;

	memcpy(&definition.fieldKey, &identifier, sizeof(identifier));
	definition.matchType = matchType;
	definition.conditionValue.type = FWP_V4_ADDR_MASK;

	//
	// Goal, buffer with:
	//
	// FWPM_FILTER_CONDITION0
	// FWP_V4_ADDR_AND_MASK
	//

	auto composer = ::common::BinaryComposer
	{
		ConstBufferView(&definition, sizeof(definition)),
		ConstBufferView(&network, sizeof(network))
	};

	auto buffer = composer.buffer();
	auto offsets = composer.offsets();

	//
	// Correct stored pointer
	//

	auto finalDefinition = MAKE_PTR(buffer, offsets[0], FWPM_FILTER_CONDITION0);
	auto finalNetwork = MAKE_PTR(buffer, offsets[1], FWP_V4_ADDR_AND_MASK);

	finalDefinition->conditionValue.v4AddrMask = finalNetwork;

	return composer.acquire();
}

//static
Buffer ConditionAssembler::Ipv6Network(
	const GUID &identifier,
	FWP_MATCH_TYPE matchType,
	const FWP_V6_ADDR_AND_MASK &network)
{
	FWPM_FILTER_CONDITION0 definition;

	memcpy(&definition.fieldKey, &identifier, sizeof(identifier));
	definition.matchType = matchType;
	definition.conditionValue.type = FWP_V6_ADDR_MASK;

	//
	// Goal, buffer with:
	//
	// FWPM_FILTER_CONDITION0
	// FWP_V6_ADDR_AND_MASK
	//

	auto composer = ::common::BinaryComposer
	{
		ConstBufferView(&definition, sizeof(definition)),
		ConstBufferView(&network, sizeof(network))
	};

	auto buffer = composer.buffer();
	auto offsets = composer.offsets();

	//
	// Correct stored pointer
	//

	auto finalDefinition = MAKE_PTR(buffer, offsets[0], FWPM_FILTER_CONDITION0);
	auto finalNetwork = MAKE_PTR(buffer, offsets[1], FWP_V6_ADDR_AND_MASK);

	finalDefinition->conditionValue.v6AddrMask = finalNetwork;

	return composer.acquire();
}

}
