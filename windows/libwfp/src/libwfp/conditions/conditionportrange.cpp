#include "stdafx.h"
#include "conditionportrange.h"
#include <libwfp/internal/conditionassembler.h>
#include <sstream>

using ConditionAssembler = ::wfp::internal::ConditionAssembler;

namespace wfp::conditions {

ConditionPortRange::ConditionPortRange(bool local, uint16_t firstPort, uint16_t lastPort, const IStrictComparison &comparison)
	: m_local(local)
	, m_firstPort(firstPort)
	, m_lastPort(lastPort)
	, m_comparison(comparison)
{
	FWP_RANGE0 range;

	range.valueLow.type = FWP_UINT16;
	range.valueLow.uint16 = m_firstPort;
	range.valueHigh.type = FWP_UINT16;
	range.valueHigh.uint16 = m_lastPort;

	m_assembled = ConditionAssembler::Range(identifier(), m_comparison.op(), range);
}

//static
std::unique_ptr<ConditionPortRange> ConditionPortRange::Local(uint16_t firstPort, uint16_t lastPort, const IStrictComparison &comparison)
{
	return std::unique_ptr<ConditionPortRange>(new ConditionPortRange(true, firstPort, lastPort, comparison));
}

//static
std::unique_ptr<ConditionPortRange> ConditionPortRange::Remote(uint16_t firstPort, uint16_t lastPort, const IStrictComparison &comparison)
{
	return std::unique_ptr<ConditionPortRange>(new ConditionPortRange(false, firstPort, lastPort, comparison));
}

std::wstring ConditionPortRange::toString() const
{
	std::wstringstream ss;

	const auto target = (m_local ? L"local" : L"remote");

	ss << target << L" port " << (m_comparison.op() == FWP_MATCH_EQUAL ? L"inside" : L"outside")
		<< L" range " << m_firstPort << L"-" << m_lastPort;

	return ss.str();
}

const GUID &ConditionPortRange::identifier() const
{
	return (m_local ? FWPM_CONDITION_IP_LOCAL_PORT : FWPM_CONDITION_IP_REMOTE_PORT);
}

const FWPM_FILTER_CONDITION0 &ConditionPortRange::condition() const
{
	return *reinterpret_cast<FWPM_FILTER_CONDITION0 *>(m_assembled.data());
}

}
