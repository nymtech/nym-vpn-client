#pragma once

#include <string>
#include <windows.h>

namespace common::registry
{

class RegistryPath
{
public:

	RegistryPath(const std::wstring &path);

	RegistryPath(HKEY key, const std::wstring &subkey)
		: m_key(key)
		, m_subkey(subkey)
	{
	}

	HKEY key() const
	{
		return m_key;
	}

	const std::wstring &subkey() const
	{
		return m_subkey;
	}

private:

	HKEY m_key;
	std::wstring m_subkey;
};

}
