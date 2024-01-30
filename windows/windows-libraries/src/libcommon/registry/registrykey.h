#pragma once

#include <cstdint>
#include <string>
#include <vector>
#include <functional>
#include <windows.h>

namespace common::registry
{

class RegistryKey
{
public:

	RegistryKey(HKEY key);
	~RegistryKey();

	RegistryKey(const RegistryKey &rhs) = delete;
	RegistryKey &operator=(const RegistryKey &rhs) = delete;

	enum class ValueStringType
	{
		RegularString = REG_SZ,
		ExpandableString = REG_EXPAND_SZ,
	};

	void flush();

	void writeValue(const std::wstring &valueName, const std::wstring &valueData, ValueStringType type = ValueStringType::RegularString);
	void writeValue(const std::wstring &valueName, uint32_t valueData);
	void writeValue(const std::wstring &valueName, uint64_t valueData);
	void writeValue(const std::wstring &valueName, const std::vector<uint8_t> &valueData);
	void writeValue(const std::wstring &valueName, const std::vector<std::wstring> &valueData);

	void deleteValue(const std::wstring &valueName);

	std::wstring readString(const std::wstring &valueName, ValueStringType type = ValueStringType::RegularString) const;
	uint32_t readUint32(const std::wstring &valueName) const;
	uint64_t readUint64(const std::wstring &valueName) const;
	std::vector<uint8_t> readBinaryBlob(const std::wstring &valueName) const;
	std::vector<std::wstring> readStringArray(const std::wstring &valueName) const;

	void enumerateSubKeys(std::function<bool(const std::wstring &keyName)> callback);
	void enumerateValues(std::function<bool(const std::wstring &valueName, uint32_t valueType)> callback);

private:

	HKEY m_key;

	std::vector<uint8_t> readRaw(const std::wstring &valueName, DWORD dataType) const;
};

}
