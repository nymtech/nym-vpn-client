#include "stdafx.h"
#include "conditionicmp.h"
#include <libwfp/internal/conditionassembler.h>
#include <libcommon/error.h>
#include <sstream>

using ConditionAssembler = ::wfp::internal::ConditionAssembler;

namespace wfp::conditions {

ConditionIcmp::ConditionIcmp(ConditionType type, uint16_t value, const IRelaxedComparison &comparison)
	: m_type(type)
	, m_value(value)
	, m_comparison(comparison)
{
	m_assembled = ConditionAssembler::Uint16(identifier(), comparison.op(), value);
}

std::wstring ConditionIcmp::toString() const
{
	std::wstringstream ss;

	switch (m_type)
	{
		case ConditionType::Type:
		{
			ss << L"icmp type";
			break;
		}
		case ConditionType::Code:
		{
			ss << L"icmp code";
			break;
		}
		default:
		{
			THROW_ERROR("Missing case handler in switch clause");
		}
	}

	ss << L" " << m_comparison.toString() << L" " << m_value;

	return ss.str();
}

const GUID &ConditionIcmp::identifier() const
{
	switch (m_type)
	{
		case ConditionType::Type:
		{
			return FWPM_CONDITION_ICMP_TYPE;
		}
		case ConditionType::Code:
		{
			return FWPM_CONDITION_ICMP_CODE;
		}
		default:
		{
			THROW_ERROR("Missing case handler in switch clause");
		}
	}
}

const FWPM_FILTER_CONDITION0 &ConditionIcmp::condition() const
{
	return *reinterpret_cast<FWPM_FILTER_CONDITION0 *>(m_assembled.data());
}

//static
std::unique_ptr<ConditionIcmp> ConditionIcmp::Type(uint16_t value, const IRelaxedComparison &comparison)
{
	return std::unique_ptr<ConditionIcmp>(new ConditionIcmp(ConditionType::Type, value, comparison));
}

//static
std::unique_ptr<ConditionIcmp> ConditionIcmp::Code(uint16_t value, const IRelaxedComparison &comparison)
{
	return std::unique_ptr<ConditionIcmp>(new ConditionIcmp(ConditionType::Code, value, comparison));
}

}
