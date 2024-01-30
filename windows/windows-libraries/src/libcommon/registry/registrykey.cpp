#include "stdafx.h"
#include "registrykey.h"
#include <libcommon/error.h>

namespace common::registry
{

RegistryKey::RegistryKey(HKEY key)
	: m_key(key)
{
}

RegistryKey::~RegistryKey()
{
	RegCloseKey(m_key);
}

void RegistryKey::flush()
{
	const auto status = RegFlushKey(m_key);

	if (ERROR_SUCCESS != status)
	{
		THROW_WINDOWS_ERROR(status, "Flush registry key");
	}
}

void RegistryKey::writeValue(const std::wstring &valueName, const std::wstring &valueData, ValueStringType type)
{
	auto dataLength = static_cast<DWORD>((valueData.size() + 1) * sizeof(wchar_t));

	const auto status = RegSetValueExW(m_key, valueName.c_str(), 0, static_cast<DWORD>(type),
		reinterpret_cast<const BYTE *>(valueData.c_str()), dataLength);

	if (ERROR_SUCCESS != status)
	{
		THROW_WINDOWS_ERROR(status, "Write registry value");
	}
}

void RegistryKey::writeValue(const std::wstring &valueName, uint32_t valueData)
{
	const auto status = RegSetValueExW(m_key, valueName.c_str(), 0, REG_DWORD,
		reinterpret_cast<const BYTE *>(&valueData), sizeof(uint32_t));

	if (ERROR_SUCCESS != status)
	{
		THROW_WINDOWS_ERROR(status, "Write registry value");
	}
}

void RegistryKey::writeValue(const std::wstring &valueName, uint64_t valueData)
{
	const auto status = RegSetValueExW(m_key, valueName.c_str(), 0, REG_QWORD,
		reinterpret_cast<const BYTE *>(&valueData), sizeof(uint64_t));

	if (ERROR_SUCCESS != status)
	{
		THROW_WINDOWS_ERROR(status, "Write registry value");
	}
}

void RegistryKey::writeValue(const std::wstring &valueName, const std::vector<uint8_t> &valueData)
{
	auto dataLength = static_cast<DWORD>(valueData.size());

	const auto status = RegSetValueExW(m_key, valueName.c_str(), 0, REG_BINARY,
		&valueData[0], dataLength);

	if (ERROR_SUCCESS != status)
	{
		THROW_WINDOWS_ERROR(status, "Write registry value");
	}
}

void RegistryKey::writeValue(const std::wstring &valueName, const std::vector<std::wstring> &valueData)
{
	size_t requiredSize = 0;

	for (const auto &value : valueData)
	{
		requiredSize += (value.size() + 1);
	}

	// Terminating null byte.
	++requiredSize;

	std::vector<wchar_t> buffer(requiredSize);
	wchar_t *dest = &buffer[0];

	for (const auto &value : valueData)
	{
		size_t segmentSize = (value.size() + 1);

		memcpy(dest, value.c_str(), segmentSize * sizeof(wchar_t));
		dest += segmentSize;
	}

	*dest = L'\x0';

	//
	// Update registry.
	//

	auto dataLength = static_cast<DWORD>(buffer.size() * sizeof(wchar_t));

	const auto status = RegSetValueExW(m_key, valueName.c_str(), 0, REG_MULTI_SZ,
		reinterpret_cast<const BYTE *>(&buffer[0]), dataLength);

	if (ERROR_SUCCESS != status)
	{
		THROW_WINDOWS_ERROR(status, "Write registry value");
	}
}

void RegistryKey::deleteValue(const std::wstring &valueName)
{
	const auto status = RegDeleteValueW(m_key, valueName.c_str());

	if (ERROR_SUCCESS != status)
	{
		THROW_WINDOWS_ERROR(status, "Delete registry value");
	}
}

std::wstring RegistryKey::readString(const std::wstring &valueName, ValueStringType type) const
{
	auto buffer = readRaw(valueName, static_cast<DWORD>(type));

	if (buffer.size() <= 1)
	{
		return std::wstring();
	}

	auto begin = reinterpret_cast<const wchar_t *>(&buffer[0]);

	//
	// There could be zero, one or more null bytes at the end.
	// Scan backwards until we find a non-null wchar_t.
	//
	// Dividing by sizeof(wchar_t) discards any odd byte at the end.
	//

	auto lastChar = begin + (buffer.size() / sizeof(wchar_t)) - 1;

	while (lastChar >= begin && L'\x0' == *lastChar)
	{
		--lastChar;
	}

	if (lastChar < begin)
	{
		return std::wstring();
	}

	return std::wstring(begin, lastChar + 1);
}

uint32_t RegistryKey::readUint32(const std::wstring &valueName) const
{
	auto buffer = readRaw(valueName, REG_DWORD);

	return *reinterpret_cast<const uint32_t *>(&buffer[0]);
}

uint64_t RegistryKey::readUint64(const std::wstring &valueName) const
{
	auto buffer = readRaw(valueName, REG_QWORD);

	return *reinterpret_cast<const uint64_t *>(&buffer[0]);
}

std::vector<uint8_t> RegistryKey::readBinaryBlob(const std::wstring &valueName) const
{
	return readRaw(valueName, REG_BINARY);
}

std::vector<std::wstring> RegistryKey::readStringArray(const std::wstring &valueName) const
{
	auto buffer = readRaw(valueName, REG_MULTI_SZ);

	if (buffer.size() <= 1)
	{
		return std::vector<std::wstring>();
	}

	std::vector<std::wstring> result;

	// Dividing by sizeof(wchar_t) discards any odd byte at the end.
	auto end = reinterpret_cast<const wchar_t *>(&buffer[0]) + (buffer.size() / sizeof(wchar_t));

	auto valueStart = reinterpret_cast<const wchar_t *>(&buffer[0]);

	for (;;)
	{
		if (valueStart == end)
		{
			break;
		}

		auto valueEnd = valueStart;

		while (valueEnd != end && L'\x0' != *valueEnd)
		{
			++valueEnd;
		}

		if (valueEnd - valueStart > 0)
		{
			result.emplace_back(std::wstring(valueStart, valueEnd));
		}

		valueStart = valueEnd;

		while (valueStart != end && L'\x0' == *valueStart)
		{
			++valueStart;
		}
	}

	return result;
}

void RegistryKey::enumerateSubKeys(std::function<bool(const std::wstring &)> callback)
{
	DWORD longestSubKeyName = 0;

	const auto queryStatus = RegQueryInfoKeyW(m_key, nullptr, nullptr, nullptr, nullptr,
		&longestSubKeyName, nullptr, nullptr, nullptr, nullptr, nullptr, nullptr);

	if (ERROR_SUCCESS != queryStatus)
	{
		THROW_WINDOWS_ERROR(queryStatus, "Query for longest subkey name length");
	}

	std::vector<wchar_t> buffer(longestSubKeyName + 1);

	for (DWORD subKeyIndex = 0;;)
	{
		DWORD subKeyNameLength = static_cast<DWORD>(buffer.size());

		const auto enumStatus = RegEnumKeyExW(m_key, subKeyIndex, &buffer[0], &subKeyNameLength, nullptr, nullptr, nullptr, nullptr);

		if (ERROR_NO_MORE_ITEMS == enumStatus)
		{
			break;
		}

		//
		// There's a race condition in effect between determining the required buffer size
		// and reading out the longest key name.
		//
		if (ERROR_MORE_DATA == enumStatus)
		{
			buffer.resize(buffer.size() * 2);
			continue;
		}

		if (ERROR_SUCCESS != enumStatus)
		{
			THROW_WINDOWS_ERROR(enumStatus, "Enumerate next subkey");
		}

		if (false == callback(std::wstring(&buffer[0])))
		{
			return;
		}

		++subKeyIndex;
	}
}

void RegistryKey::enumerateValues(std::function<bool(const std::wstring &, uint32_t)> callback)
{
	DWORD longestValueName = 0;

	const auto queryStatus = RegQueryInfoKeyW(m_key, nullptr, nullptr, nullptr, nullptr,
		nullptr, nullptr, nullptr, &longestValueName, nullptr, nullptr, nullptr);

	if (ERROR_SUCCESS != queryStatus)
	{
		THROW_WINDOWS_ERROR(queryStatus, "Query for longest value name length");
	}

	std::vector<wchar_t> buffer(longestValueName + 1);

	for (DWORD valueIndex = 0;;)
	{
		DWORD valueNameLength = static_cast<DWORD>(buffer.size());
		DWORD valueType;

		const auto enumStatus = RegEnumValueW(m_key, valueIndex, &buffer[0], &valueNameLength, nullptr, &valueType, nullptr, nullptr);

		if (ERROR_NO_MORE_ITEMS == enumStatus)
		{
			break;
		}

		//
		// There's a race condition in effect between determining the required buffer size
		// and reading out the longest value name.
		//
		if (ERROR_MORE_DATA == enumStatus)
		{
			buffer.resize(buffer.size() * 2);
			continue;
		}

		if (ERROR_SUCCESS != enumStatus)
		{
			THROW_WINDOWS_ERROR(enumStatus, "Enumerate next value");
		}

		if (false == callback(std::wstring(&buffer[0]), valueType))
		{
			return;
		}

		++valueIndex;
	}
}

std::vector<uint8_t> RegistryKey::readRaw(const std::wstring &valueName, DWORD dataType) const
{
	DWORD actualDataType;
	DWORD dataSize = 0;

	auto status = RegQueryValueExW(m_key, valueName.c_str(), nullptr, &actualDataType, nullptr, &dataSize);

	if (ERROR_SUCCESS != status)
	{
		THROW_WINDOWS_ERROR(status, "Query for registry value data type and data length");
	}

	if (actualDataType != dataType)
	{
		THROW_ERROR("Unexpected registry value data type");
	}

	if (0 == dataSize)
	{
		return std::vector<uint8_t>();
	}

	std::vector<uint8_t> buffer(dataSize);

	status = RegQueryValueExW(m_key, valueName.c_str(), nullptr, nullptr, &buffer[0], &dataSize);

	if (ERROR_SUCCESS != status)
	{
		THROW_WINDOWS_ERROR(status, "Read registry value");
	}

	return buffer;
}

}
