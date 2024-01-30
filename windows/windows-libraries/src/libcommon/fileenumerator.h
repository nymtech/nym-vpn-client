#pragma once

#include <string>
#include <memory>
#include <vector>
#include <windows.h>

namespace common::fs
{

//
// Note on using filters: Only those candidates that are matched by the filter are kept.
//
struct IFileEnumeratorFilter
{
	virtual ~IFileEnumeratorFilter()
	{
	}

	virtual bool match(const WIN32_FIND_DATAW &candidate) = 0;
};

struct FilterDirectories : public IFileEnumeratorFilter
{
	bool match(const WIN32_FIND_DATAW &candidate) override
	{
		return 0 != (candidate.dwFileAttributes & FILE_ATTRIBUTE_DIRECTORY);
	}
};

struct FilterFiles : public FilterDirectories
{
	bool match(const WIN32_FIND_DATAW &candidate) override
	{
		return !FilterDirectories::match(candidate);
	}
};

struct FilterNotRelativeDirs : public IFileEnumeratorFilter
{
	bool match(const WIN32_FIND_DATAW &candidate) override
	{
		return 0 != wcscmp(candidate.cFileName, L".")
			&& 0 != wcscmp(candidate.cFileName, L"..");
	}
};

class FilterNamedSet : public IFileEnumeratorFilter
{
public:

	FilterNamedSet()
	{
	}

	//
	// Convenience only.
	//
	FilterNamedSet(std::wstring &&object)
	{
		addObject(std::move(object));
	}

	void addObject(std::wstring &&object)
	{
		m_objects.emplace_back(std::move(object));
	}

	bool match(const WIN32_FIND_DATAW &candidate) override
	{
		for (const auto &object : m_objects)
		{
			if (0 == _wcsicmp(candidate.cFileName, object.c_str()))
			{
				return true;
			}
		}

		return false;
	}

private:

	std::vector<std::wstring> m_objects;
};

struct FilterNotNamedSet : public FilterNamedSet
{
	FilterNotNamedSet()
	{
	}

	//
	// Convenience only.
	//
	FilterNotNamedSet(std::wstring &&object)
	{
		addObject(std::move(object));
	}

	bool match(const WIN32_FIND_DATAW &candidate) override
	{
		return !FilterNamedSet::match(candidate);
	}
};

class FileEnumerator
{
public:

	FileEnumerator(const std::wstring &directory);
	FileEnumerator(const std::wstring &directory, const std::wstring objectMask);

	FileEnumerator(const FileEnumerator &) = delete;
	FileEnumerator &operator=(const FileEnumerator &) = delete;

	~FileEnumerator();

	void addFilter(std::unique_ptr<IFileEnumeratorFilter> &&filter)
	{
		m_filters.emplace_back(std::move(filter));
	}

	bool next(WIN32_FIND_DATAW &found);

	const std::wstring &getDirectory() const
	{
		return m_directory;
	};

private:

	bool m_exhausted = true;
	bool m_haveCachedEntry = false;

	const std::wstring m_directory;

	HANDLE m_enumHandle;

	WIN32_FIND_DATAW m_cached;

	std::vector<std::unique_ptr<IFileEnumeratorFilter> > m_filters;

	bool match(const WIN32_FIND_DATAW candidate);
};

}
