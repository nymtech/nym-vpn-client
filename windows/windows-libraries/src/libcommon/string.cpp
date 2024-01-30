#include "stdafx.h"
#include <ip2string.h>
#include <mstcpip.h>
#include <ws2ipdef.h>
#include "string.h"
#include "memory.h"
#include "error.h"
#include <algorithm>
#include <iomanip>
#include <optional>
#include <memory>
#include <sddl.h>
#include <sstream>
#include <wchar.h>

namespace {

constexpr size_t MAX_IPV4_STRING_LENGTH = 16;
constexpr size_t MAX_IPV6_STRING_LENGTH = 46;

std::vector<uint8_t> ToMultiByte(const std::wstring &str, UINT codePage, bool throwOnError)
{
	if (str.empty())
	{
		return std::vector<uint8_t>(1);
	}

	const size_t mbStringSize = static_cast<size_t>(WideCharToMultiByte(
		codePage,
		0,
		str.c_str(),
		-1,
		nullptr,
		0,
		nullptr,
		nullptr
	));

	if (0 == mbStringSize)
	{
		if (throwOnError)
		{
			THROW_WINDOWS_ERROR(GetLastError(), "WideCharToMultiByte");
		}
		return std::vector<uint8_t>(1);
	}

	std::vector<uint8_t> buffer(mbStringSize);

	if (0 == WideCharToMultiByte(
		codePage,
		0,
		str.c_str(),
		-1,
		reinterpret_cast<char *>(buffer.data()),
		static_cast<int>(mbStringSize),
		nullptr,
		nullptr
	))
	{
		if (throwOnError)
		{
			THROW_WINDOWS_ERROR(GetLastError(), "WideCharToMultiByte");
		}
		return std::vector<uint8_t>(1);
	}

	return buffer;
}

} // anonymous namespace

namespace common::string {

std::wstring FormatGuid(const GUID &guid)
{
	LPOLESTR buffer;

	auto status = StringFromCLSID(guid, &buffer);

	if (status != S_OK)
	{
		THROW_ERROR("Failed to format GUID");
	}

	std::wstring formatted(buffer);

	CoTaskMemFree(buffer);

	return formatted;
}

std::wstring FormatSid(const SID &sid)
{
	LPWSTR buffer;

	auto status = ConvertSidToStringSidW(const_cast<SID *>(&sid), &buffer);

	if (0 == status)
	{
		THROW_ERROR("Failed to format SID");
	}

	std::wstring formatted(buffer);

	LocalFree((HLOCAL)buffer);

	return formatted;
}

std::wstring Join(const std::vector<std::wstring> &parts, const std::wstring &delimiter)
{
	switch (parts.size())
	{
	case 0:
		return L"";
	case 1:
		return parts[0];
	default:
		{
			std::wstring joined;

			size_t reserveSize = 0;

			std::for_each(parts.begin(), parts.end(), [&reserveSize, &delimiter](const std::wstring &part)
			{
				reserveSize += (part.size() + delimiter.size());
			});

			joined.reserve(reserveSize);

			std::for_each(parts.begin(), parts.end(), [&joined, &delimiter](const std::wstring &part)
			{
				if (!joined.empty())
				{
					joined.append(delimiter);
				}

				joined.append(part);
			});

			return joined;
		}
	};
}

template<>
std::wstring FormatIpv4<AddressOrder::NetworkByteOrder>(uint32_t ip)
{
	in_addr addr;
	addr.S_un.S_addr = ip;

	std::vector<wchar_t> ipString(MAX_IPV4_STRING_LENGTH + 1);
	RtlIpv4AddressToStringW(&addr, ipString.data());

	return ipString.data();
}

template<>
std::wstring FormatIpv4<AddressOrder::HostByteOrder>(uint32_t ip)
{
	return FormatIpv4<AddressOrder::NetworkByteOrder>(htonl(ip));
}

std::wstring FormatIpv6(const uint8_t ip[16])
{
	in6_addr addr;
	std::copy(ip, ip + 16, addr.u.Byte);

	std::vector<wchar_t> ipString(MAX_IPV6_STRING_LENGTH + 1);
	RtlIpv6AddressToStringW(&addr, ipString.data());

	return ipString.data();
}

std::wstring FormatIpv6(const uint8_t ip[16], uint8_t routingPrefix)
{
	std::wstringstream ss;

	ss << FormatIpv6(ip) << L"/" << routingPrefix;

	return ss.str();
}

std::wstring FormatTime(const FILETIME &filetime)
{
	FILETIME ft2;

	if (FALSE == FileTimeToLocalFileTime(&filetime, &ft2))
	{
		THROW_ERROR("Failed to convert time");
	}

	return FormatLocalTime(ft2);
}

std::wstring FormatLocalTime(const FILETIME &filetime)
{
	SYSTEMTIME st;

	if (FALSE == FileTimeToSystemTime(&filetime, &st))
	{
		THROW_ERROR("Failed to convert time");
	}

	std::wstringstream ss;

	ss << st.wYear << L'-'
		<< std::setw(2) << std::setfill(L'0') << st.wMonth << L'-'
		<< std::setw(2) << std::setfill(L'0') << st.wDay << L' '
		<< std::setw(2) << std::setfill(L'0') << st.wHour << L':'
		<< std::setw(2) << std::setfill(L'0') << st.wMinute << L':'
		<< std::setw(2) << std::setfill(L'0') << st.wSecond;

	return ss.str();
}

std::wstring Lower(const std::wstring &str)
{
	auto bufferSize = str.size() + 1;

	auto buffer = std::make_unique<wchar_t[]>(bufferSize);
	wcscpy_s(buffer.get(), bufferSize, str.c_str());

	_wcslwr_s(buffer.get(), bufferSize);

	return buffer.get();
}

std::vector<std::wstring> Tokenize(const std::wstring &str, const std::wstring &delimiters)
{
	auto bufferSize = str.size() + 1;

	auto buffer = std::make_unique<wchar_t[]>(bufferSize);
	wcscpy_s(buffer.get(), bufferSize, str.c_str());

	wchar_t *context = nullptr;

	auto token = wcstok_s(buffer.get(), delimiters.c_str(), &context);

	std::vector<std::wstring> tokens;

	while (token != nullptr)
	{
		tokens.push_back(token);
		token = wcstok_s(nullptr, delimiters.c_str(), &context);
	}

	return tokens;
}

std::vector<uint8_t> ToUtf8(const std::wstring &str, bool throwOnError)
{
	return ToMultiByte(str, CP_UTF8, throwOnError);
}

std::string ToAnsi(const std::wstring &str, bool throwOnError)
{
	const auto bytes = ToMultiByte(str, CP_ACP, throwOnError);
	return std::string(reinterpret_cast<const char*>(bytes.data()));
}

std::wstring ToWide(const std::string &str, bool throwOnError)
{
	if (str.empty())
	{
		return std::wstring();
	}

	const auto wsLength = static_cast<size_t>(MultiByteToWideChar(
		CP_ACP,
		0,
		str.c_str(),
		-1,
		nullptr,
		0
	));

	if (0 == wsLength)
	{
		if (throwOnError)
		{
			THROW_WINDOWS_ERROR(GetLastError(), "MultiByteToWideChar");
		}
		return std::wstring();
	}

	std::vector<wchar_t> buffer(wsLength);

	if (0 == MultiByteToWideChar(
		CP_ACP,
		0,
		str.c_str(),
		-1,
		buffer.data(),
		static_cast<int>(wsLength)
	))
	{
		if (throwOnError)
		{
			THROW_WINDOWS_ERROR(GetLastError(), "MultiByteToWideChar");
		}
		return std::wstring();
	}

	return buffer.data();
}

std::wstring Summary(const std::wstring &str, size_t max)
{
	if (str.size() <= max)
	{
		return str;
	}

	const wchar_t *padding = L"...";
	const size_t paddingLength = 3;

	if (max < paddingLength)
	{
		THROW_ERROR("Requested summary is too short");
	}

	auto summary = str.substr(0, max - paddingLength);
	summary.append(padding);

	return summary;
}

KeyValuePairs SplitKeyValuePairs(const std::vector<std::wstring> &serializedPairs)
{
	KeyValuePairs result;

	for (const auto &pair : serializedPairs)
	{
		auto index = pair.find(L'=');

		if (index == std::wstring::npos)
		{
			// Insert key with empty value.
			result.insert(std::make_pair(pair, L""));
		}
		else
		{
			result.insert(std::make_pair(
				pair.substr(0, index),
				pair.substr(index + 1)
			));
		}
	}

	return result;
}

const char *TrimChars = "\r\n\t ";
const wchar_t *WideTrimChars = L"\r\n\t ";

}
