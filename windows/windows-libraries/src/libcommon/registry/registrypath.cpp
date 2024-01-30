#include "stdafx.h"
#include "registrypath.h"
#include <libcommon/string.h>
#include <libcommon/error.h>

namespace common::registry
{

RegistryPath::RegistryPath(const std::wstring &path)
{
	const auto pathTokens = common::string::Tokenize(path, L"/\\");

	if (pathTokens.size() < 2)
	{
		THROW_ERROR("Invalid registry path");
	}

	const auto keyName = pathTokens[0];

	if (0 == _wcsicmp(keyName.c_str(), L"HKEY_CLASSES_ROOT")
		|| 0 == _wcsicmp(keyName.c_str(), L"HKCR"))
	{
		m_key = HKEY_CLASSES_ROOT;
	}
	else if (0 == _wcsicmp(keyName.c_str(), L"HKEY_CURRENT_USER")
		|| 0 == _wcsicmp(keyName.c_str(), L"HKCU"))
	{
		m_key = HKEY_CURRENT_USER;
	}
	else if (0 == _wcsicmp(keyName.c_str(), L"HKEY_LOCAL_MACHINE")
		|| 0 == _wcsicmp(keyName.c_str(), L"HKLM"))
	{
		m_key = HKEY_LOCAL_MACHINE;
	}
	else if (0 == _wcsicmp(keyName.c_str(), L"HKEY_USERS")
		|| 0 == _wcsicmp(keyName.c_str(), L"HKU"))
	{
		m_key = HKEY_USERS;
	}
	else if (0 == _wcsicmp(keyName.c_str(), L"HKEY_PERFORMANCE_DATA")
		|| 0 == _wcsicmp(keyName.c_str(), L"HKPD"))
	{
		m_key = HKEY_PERFORMANCE_DATA;
	}
	else if (0 == _wcsicmp(keyName.c_str(), L"HKEY_PERFORMANCE_TEXT")
		|| 0 == _wcsicmp(keyName.c_str(), L"HKPT"))
	{
		m_key = HKEY_PERFORMANCE_TEXT;
	}
	else if (0 == _wcsicmp(keyName.c_str(), L"HKEY_PERFORMANCE_NLSTEXT")
		|| 0 == _wcsicmp(keyName.c_str(), L"HKPNLST"))
	{
		m_key = HKEY_PERFORMANCE_NLSTEXT;
	}
	else if (0 == _wcsicmp(keyName.c_str(), L"HKEY_CURRENT_CONFIG")
		|| 0 == _wcsicmp(keyName.c_str(), L"HKCC"))
	{
		m_key = HKEY_CURRENT_CONFIG;
	}
	else if (0 == _wcsicmp(keyName.c_str(), L"HKEY_DYN_DATA")
		|| 0 == _wcsicmp(keyName.c_str(), L"HKDD"))
	{
		m_key = HKEY_DYN_DATA;
	}
	else if (0 == _wcsicmp(keyName.c_str(), L"HKEY_CURRENT_USER_LOCAL_SETTINGS")
		|| 0 == _wcsicmp(keyName.c_str(), L"HKCULS"))
	{
		m_key = HKEY_CURRENT_USER_LOCAL_SETTINGS;
	}
	else
	{
		THROW_ERROR("Invalid registry root");
	}

	//
	// Merge subkey path back together.
	//

	std::wstring subkey;

	std::for_each(++pathTokens.begin(), pathTokens.end(), [&subkey](const std::wstring &segment)
	{
		if (!subkey.empty())
		{
			subkey.append(L"\\");
		}

		subkey.append(segment);
	});

	m_subkey = subkey;
}

}
