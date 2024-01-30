#include "stdafx.h"
#include "conditiondirection.h"
#include <libwfp/internal/conditionassembler.h>

using ConditionAssembler = ::wfp::internal::ConditionAssembler;

namespace wfp::conditions {

ConditionDirection::ConditionDirection(Direction direction)
	: m_direction(direction)
{
	m_assembled = ConditionAssembler::Uint32
	(
		identifier(),
		FWP_MATCH_EQUAL,
		static_cast<UINT32>(Direction::Inbound == m_direction ? FWP_DIRECTION_INBOUND : FWP_DIRECTION_OUTBOUND)
	);
}

std::wstring ConditionDirection::toString() const
{
	return
	(
		Direction::Inbound == m_direction
		? L"direction = inbound"
		: L"direction = outbound"
	);
}

const GUID &ConditionDirection::identifier() const
{
	return FWPM_CONDITION_DIRECTION;
}

const FWPM_FILTER_CONDITION0 &ConditionDirection::condition() const
{
	return *reinterpret_cast<FWPM_FILTER_CONDITION0 *>(m_assembled.data());
}

//static
std::unique_ptr<ConditionDirection> ConditionDirection::Inbound()
{
	return std::unique_ptr<ConditionDirection>(new ConditionDirection(Direction::Inbound));
}

//static
std::unique_ptr<ConditionDirection> ConditionDirection::Outbound()
{
	return std::unique_ptr<ConditionDirection>(new ConditionDirection(Direction::Outbound));
}

}
