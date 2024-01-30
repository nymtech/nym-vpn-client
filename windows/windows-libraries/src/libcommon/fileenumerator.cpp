#include "stdafx.h"
#include "fileenumerator.h"
#include "filesystem.h"
#include <filesystem>

namespace common::fs
{

FileEnumerator::FileEnumerator(const std::wstring &directory)
	: FileEnumerator(directory, L"*")
{
}

FileEnumerator::FileEnumerator(const std::wstring &directory, const std::wstring objectMask)
	: m_directory(directory)
{
	const auto findspec = std::filesystem::path(directory).append(objectMask);

	m_enumHandle = FindFirstFileW
	(
		std::wstring(L"\\\\?\\").append(findspec).c_str(),
		&m_cached
	);

	if (m_enumHandle != INVALID_HANDLE_VALUE)
	{
		m_exhausted = false;
		m_haveCachedEntry = true;
	}
}

FileEnumerator::~FileEnumerator()
{
	if (m_enumHandle != INVALID_HANDLE_VALUE)
	{
		FindClose(m_enumHandle);
	}
}

bool FileEnumerator::next(WIN32_FIND_DATAW &found)
{
	if (m_exhausted)
	{
		return false;
	}

	if (m_haveCachedEntry)
	{
		m_haveCachedEntry = false;

		if (match(m_cached))
		{
			memcpy(&found, &m_cached, sizeof(found));
			return true;
		}

		//
		// Fall through.
		//
	}

	while (FALSE != FindNextFileW(m_enumHandle, &found))
	{
		if (match(found))
		{
			return true;
		}
	}

	m_exhausted = true;
	return false;
}

bool FileEnumerator::match(const WIN32_FIND_DATAW candidate)
{
	for (const auto &filter : m_filters)
	{
		if (false == filter->match(candidate))
		{
			return false;
		}
	}

	return true;
}

}
