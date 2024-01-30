#pragma once

#include <string>
#include <functional>
#include <unordered_set>
#include <wchar.h>
#include <windows.h>

namespace common::process
{

struct DefaultProcessNameComparator
{
	bool operator()(const std::wstring &lhs, const std::wstring &rhs) const
	{
		return 0 == _wcsicmp(lhs.c_str(), rhs.c_str());
	}
};

//
// You may want to enable SE_DEBUG_NAME before calling this function,
// since the method used involves opening a handle to each process being queried.
//
DWORD GetProcessIdFromName(const std::wstring &processName,
	std::function<bool(const std::wstring &lhs, const std::wstring &rhs)> comp = DefaultProcessNameComparator());

//
// N.B this can return an empty set.
//
std::unordered_set<DWORD> GetAllProcessIdsFromName(const std::wstring &processName,
	std::function<bool(const std::wstring &lhs, const std::wstring &rhs)> comp = DefaultProcessNameComparator());

void Run(const std::wstring &path);

void RunInContext(HANDLE securityContext, const std::wstring &path);

}
