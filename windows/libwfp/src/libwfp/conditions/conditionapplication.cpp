#include "stdafx.h"
#include "conditionapplication.h"
#include <libwfp/internal/conditionassembler.h>
#include <libcommon/error.h>
#include <sstream>

using ConditionAssembler = ::wfp::internal::ConditionAssembler;

namespace wfp::conditions {

ConditionApplication::ConditionApplication(const std::wstring &application, const IStrictComparison &comparison)
	: m_application(application)
	, m_comparison(comparison)
{
	FWP_BYTE_BLOB *blob;

	auto status = FwpmGetAppIdFromFileName0(m_application.c_str(), &blob);

	if (ERROR_SUCCESS != status)
	{
		THROW_WINDOWS_ERROR(status, "Retrieve application identifier from filename");
	}

	m_assembled = ConditionAssembler::ByteBlob(identifier(), m_comparison.op(), *blob);

	FwpmFreeMemory0((void**)&blob);
}

std::wstring ConditionApplication::toString() const
{
	std::wstringstream ss;

	ss << L"application " << m_comparison.toString() << L" " << m_application;

	return ss.str();
}

const GUID &ConditionApplication::identifier() const
{
	return FWPM_CONDITION_ALE_APP_ID;
}

const FWPM_FILTER_CONDITION0 &ConditionApplication::condition() const
{
	return *reinterpret_cast<FWPM_FILTER_CONDITION0 *>(m_assembled.data());
}

}
