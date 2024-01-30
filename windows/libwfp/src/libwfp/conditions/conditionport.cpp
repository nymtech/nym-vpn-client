#include "stdafx.h"
#include "conditionport.h"
#include <libwfp/internal/conditionassembler.h>
#include <sstream>

using ConditionAssembler = ::wfp::internal::ConditionAssembler;

namespace wfp::conditions {

ConditionPort::ConditionPort(bool local, uint16_t port, const IRelaxedComparison &comparison)
	: m_local(local)
	, m_port(port)
	, m_comparison(comparison)
{
	m_assembled = ConditionAssembler::Uint16(identifier(), m_comparison.op(), m_port);
}

//static
std::unique_ptr<ConditionPort> ConditionPort::Local(uint16_t port, const IRelaxedComparison &comparison)
{
	return std::unique_ptr<ConditionPort>(new ConditionPort(true, port, comparison));
}

//static
std::unique_ptr<ConditionPort> ConditionPort::Remote(uint16_t port, const IRelaxedComparison &comparison)
{
	return std::unique_ptr<ConditionPort>(new ConditionPort(false, port, comparison));
}

std::wstring ConditionPort::toString() const
{
	std::wstringstream ss;

	const auto target = (m_local ? L"local" : L"remote");

	ss << target << L" port " << m_comparison.toString() << L" " << m_port;

	return ss.str();
}

const GUID &ConditionPort::identifier() const
{
	return (m_local ? FWPM_CONDITION_IP_LOCAL_PORT : FWPM_CONDITION_IP_REMOTE_PORT);
}

const FWPM_FILTER_CONDITION0 &ConditionPort::condition() const
{
	return *reinterpret_cast<FWPM_FILTER_CONDITION0 *>(m_assembled.data());
}

}
