#include "stdafx.h"
#include "conditionloopback.h"
#include <libwfp/internal/conditionassembler.h>
#include <ipifcons.h>
#include <sstream>
#include <libcommon/error.h>

using ConditionAssembler = ::wfp::internal::ConditionAssembler;

namespace wfp::conditions {

namespace
{

FWP_MATCH_TYPE GetFlagsMatchType(const ComparisonSpecification &comp)
{
	switch (comp.op())
	{
		case FWP_MATCH_EQUAL: return FWP_MATCH_FLAGS_ALL_SET;
		case FWP_MATCH_NOT_EQUAL: return FWP_MATCH_FLAGS_NONE_SET;
		default:
		{
			THROW_ERROR("Missing case handler in switch clause");
		}
	}
}

} // anonymous namespace

ConditionLoopback::ConditionLoopback(Type type, const IStrictComparison &comparison)
	: m_type(type)
	, m_comparison(comparison)
{
	switch (m_type)
	{
		case Type::LoopbackInterface:
		{
			m_assembled = ConditionAssembler::Uint32(identifier(),
				m_comparison.op(), UINT32(IF_TYPE_SOFTWARE_LOOPBACK));

			break;
		}
		case Type::LoopbackTraffic:
		{
			m_assembled = ConditionAssembler::Uint32(identifier(),
				GetFlagsMatchType(m_comparison), UINT32(FWP_CONDITION_FLAG_IS_LOOPBACK));

			break;
		}
		default:
		{
			THROW_ERROR("Missing case handler in switch clause");
		}
	}
}

std::wstring ConditionLoopback::toString() const
{
	std::wstringstream ss;

	switch (m_type)
	{
		case Type::LoopbackInterface:
		{
			ss << L"interface ";
			break;
		}
		case Type::LoopbackTraffic:
		{
			ss << L"traffic category ";
			break;
		}
		default:
		{
			THROW_ERROR("Missing case handler in switch clause");
		}
	}

	ss << m_comparison.toString() << L" loopback";

	return ss.str();
}

const GUID &ConditionLoopback::identifier() const
{
	switch (m_type)
	{
		case Type::LoopbackInterface: return FWPM_CONDITION_INTERFACE_TYPE;
		case Type::LoopbackTraffic: return FWPM_CONDITION_FLAGS;
		default:
		{
			THROW_ERROR("Missing case handler in switch clause");
		}
	}
}

const FWPM_FILTER_CONDITION0 &ConditionLoopback::condition() const
{
	return *reinterpret_cast<FWPM_FILTER_CONDITION0 *>(m_assembled.data());
}

}
