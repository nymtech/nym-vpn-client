#pragma once

#include <objbase.h>
#include <windows.h>
#include <algorithm>
#include <sstream>
#include <string>
#include <unordered_map>
#include <vector>

namespace common::string {

std::wstring FormatGuid(const GUID &guid);
std::wstring FormatSid(const SID &sid);
std::wstring Join(const std::vector<std::wstring> &parts, const std::wstring &delimiter=L", ");

template<typename T>
std::wstring FormatFlags(std::vector<std::pair<T, std::wstring> > &definitions, T flags)
{
	std::vector<std::wstring> present;
	T remaining = flags;

	std::for_each(definitions.begin(), definitions.end(), [&](const std::pair<T, std::wstring> &definition)
	{
		if ((flags & definition.first) != 0)
		{
			present.push_back(definition.second);
			remaining &= ~definition.first;
		}
	});

	if (remaining != 0)
	{
		present.push_back(L"[...]");
	}

	return Join(present);
}

enum class AddressOrder
{
	HostByteOrder,
	NetworkByteOrder
};

// Format IPv4 address.
template<AddressOrder byteOrder = AddressOrder::HostByteOrder>
std::wstring FormatIpv4(uint32_t ip);

template<AddressOrder byteOrder = AddressOrder::HostByteOrder>
std::wstring FormatIpv4(uint32_t ip, uint8_t routingPrefix)
{
	std::wstringstream ss;

	ss << FormatIpv4<byteOrder>(ip) << L"/" << routingPrefix;

	return ss.str();
}

std::wstring FormatIpv6(const uint8_t ip[16]);

// TODO: Fix later if intending to produce user facing strings.
// This is a naive implementation that always formats eight words.
std::wstring FormatIpv6(const uint8_t ip[16], uint8_t routingPrefix);

std::wstring FormatTime(const FILETIME &filetime);
std::wstring FormatLocalTime(const FILETIME &filetime);

template<typename T>
bool BeginsWith(const std::basic_string<T> &hay, const std::basic_string<T> &needle)
{
	if (hay.size() < needle.size())
	{
		return false;
	}

	return 0 == hay.compare(0, needle.size(), needle);
}

std::wstring Lower(const std::wstring &str);
std::vector<std::wstring> Tokenize(const std::wstring &str, const std::wstring &delimiters);
std::vector<uint8_t> ToUtf8(const std::wstring &str, bool throwOnError = false);
std::string ToAnsi(const std::wstring &str, bool throwOnError = false);
std::wstring ToWide(const std::string &str, bool throwOnError = false);

std::wstring Summary(const std::wstring &str, size_t max);

typedef std::unordered_map<std::wstring, std::wstring> KeyValuePairs;

KeyValuePairs SplitKeyValuePairs(const std::vector<std::wstring> &serializedPairs);

extern const char *TrimChars;
extern const wchar_t *WideTrimChars;

template<typename T>
const T *SelectTrimChars()
{
	return nullptr;
}

template<>
inline const char *SelectTrimChars<char>()
{
	return TrimChars;
}

template<>
inline const wchar_t *SelectTrimChars<wchar_t>()
{
	return WideTrimChars;
}

template<typename T>
std::basic_string<T> TrimRight(const std::basic_string<T> &str)
{
	std::basic_string<T> trimmed(str);

	auto index = trimmed.find_last_not_of(SelectTrimChars<T>());

	if (std::basic_string<T>::npos == index)
	{
		trimmed.clear();
	}
	else
	{
		trimmed.resize(index + 1);
	}

	return trimmed;
}

template<typename T>
std::basic_string<T> TrimLeft(const std::basic_string<T> &str)
{
	std::basic_string<T> trimmed(str);

	auto index = trimmed.find_first_not_of(SelectTrimChars<T>());

	if (std::basic_string<T>::npos == index)
	{
		trimmed.clear();
	}
	else
	{
		trimmed = trimmed.substr(index);
	}

	return trimmed;
}

template<typename T>
std::basic_string<T> Trim(const std::basic_string<T> &str)
{
	return TrimLeft<>(TrimRight<>(str));
}

template<typename T>
T LexicalCast(const std::wstring &s)
{
	std::wstringstream ss(s);
	T casted;

	ss >> casted;

	return casted;
}

}
