#include "pch.h"
#include "libcommon/registry/registry.h"
#include "CppUnitTest.h"
#include <algorithm>
#include <functional>
#include <iostream>
#include <string>
#include <vector>
#include <unordered_set>

using namespace Microsoft::VisualStudio::CppUnitTestFramework;

namespace Microsoft::VisualStudio::CppUnitTestFramework
{

template<typename ContainerType, wchar_t openDelim = L'[', wchar_t closeDelim = L']'>
std::wstring ContainerToString(const ContainerType &vec)
{
	std::wstring output;
	output += openDelim;
	for (const auto &elem : vec)
	{
		if (output.length() > 1)
		{
			output.append(L", ");
		}
		output.append(ToString(elem));
	}
	output += closeDelim;
	return output;
}

template<typename T>
std::wstring SetToString(const std::unordered_set<T> &collection)
{
	return ContainerToString<std::unordered_set<std::wstring>, L'{', L'}'>(collection);
}

template<>
std::wstring ToString<std::unordered_set<std::wstring>>(const std::unordered_set<std::wstring> &vec)
{
	return SetToString(vec);
}

template<>
std::wstring ToString<std::vector<std::wstring>>(const std::vector<std::wstring> &vec)
{
	return ContainerToString(vec);
}

template<>
std::wstring ToString<std::vector<uint8_t>>(const std::vector<uint8_t> &vec)
{
	return ContainerToString(vec);
}

}

namespace testlibcommon
{

using namespace common::registry;

HKEY g_regroot = HKEY_CURRENT_USER;
const wchar_t g_subkey[] = L"Software\\Amagicom-Test";

TEST_CLASS(TestLibCommonRegistry)
{
public:

	TEST_METHOD_INITIALIZE(Initialize)
	{
		Registry::CreateKey(g_regroot, g_subkey);
	}

	TEST_METHOD_CLEANUP(Cleanup)
	{
		Registry::DeleteKey(g_regroot, g_subkey);
	}

	TEST_METHOD(OpenKey)
	{
		// Read-only access.
		Registry::OpenKey(g_regroot, g_subkey);

		// Read-write access.
		Registry::OpenKey(g_regroot, g_subkey, true);
	}

	TEST_METHOD(WriteReadStringValue)
	{
		std::unique_ptr<RegistryKey> key;

		key = Registry::OpenKey(g_regroot, g_subkey, true);

		const std::wstring valueName(L"StringValue");
		const std::wstring valueData(L"waffles");

		key->writeValue(valueName, valueData);

		std::wstring readValueData;

		readValueData = key->readString(valueName);

		Assert::AreEqual(valueData, readValueData);
	}

	TEST_METHOD(WriteReadUint32Value)
	{
		std::unique_ptr<RegistryKey> key;

		key = Registry::OpenKey(g_regroot, g_subkey, true);

		const std::wstring valueName(L"Uint32Value");
		const uint32_t valueData(0xbeefcafe);

		key->writeValue(valueName, valueData);

		uint32_t readValueData;

		readValueData = key->readUint32(valueName);

		Assert::AreEqual(valueData, readValueData);
	}

	TEST_METHOD(WriteReadUint64Value)
	{
		std::unique_ptr<RegistryKey> key;

		key = Registry::OpenKey(g_regroot, g_subkey, true);

		const std::wstring valueName(L"Uint64Value");
		const uint64_t valueData(0xbeefcafebeefbabe);

		key->writeValue(valueName, valueData);

		uint64_t readValueData;

		readValueData = key->readUint64(valueName);

		Assert::AreEqual(valueData, readValueData);
	}

	TEST_METHOD(WriteReadBinaryBlobValue)
	{
		std::unique_ptr<RegistryKey> key;

		key = Registry::OpenKey(g_regroot, g_subkey, true);

		const std::wstring valueName(L"BinaryBlobValue");
		const std::vector<uint8_t> valueData
		{
			0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f, 0x10
		};

		key->writeValue(valueName, valueData);

		std::vector<uint8_t> readValueData;

		readValueData = key->readBinaryBlob(valueName);

		Assert::AreEqual(valueData, readValueData);
	}

	TEST_METHOD(WriteReadStringArrayValue)
	{
		std::unique_ptr<RegistryKey> key;

		key = Registry::OpenKey(g_regroot, g_subkey, true);

		const std::wstring valueName(L"StringArrayValue");
		const std::vector<std::wstring> valueData
		{
			L"three",
			L"blind",
			L"mice"
		};

		key->writeValue(valueName, valueData);

		std::vector<std::wstring> readValueData;

		readValueData = key->readStringArray(valueName);

		Assert::AreEqual(valueData, readValueData);
	}

	TEST_METHOD(WriteDeleteValue)
	{
		std::unique_ptr<RegistryKey> key;

		key = Registry::OpenKey(g_regroot, g_subkey, true);

		const std::wstring valueName(L"dummy");

		key->writeValue(valueName, valueName);

		key->deleteValue(valueName);
	}

	TEST_METHOD(EnumerateKeys)
	{
		const std::unordered_set<std::wstring> subkeys
		{
			L"one",
			L"two",
			L"three"
		};

		for (const auto &subkey : subkeys)
		{
			// Create subkeys to have something to enumerate
			Registry::CreateKey(g_regroot, std::wstring(g_subkey).append(L"\\").append(subkey));
		}

		// Open registry key for enumeration of subkeys
		const auto regkey = Registry::OpenKey(g_regroot, g_subkey);

		std::unordered_set<std::wstring> foundKeys;

		auto callback = [&foundKeys](const std::wstring &subkey)
		{
			foundKeys.emplace(subkey);

			// Continue enumeration.
			return true;
		};

		// Enumerate subkeys
		regkey->enumerateSubKeys(callback);

		Assert::AreEqual(
			foundKeys,
			subkeys,
			L"Set of found keys should match set of created keys"
		);
	}

	TEST_METHOD(EnumerateValues)
	{
		const std::unordered_set<std::wstring> values
		{
			L"one",
			L"two",
			L"three"
		};

		// Open registry key
		const auto regkey = Registry::OpenKey(g_regroot, g_subkey, true);

		for (const auto &value : values)
		{
			// Create registry values to have something to enumerate
			regkey->writeValue(value, L"dummy");
		}

		std::unordered_set<std::wstring> foundValues;
		std::unordered_set<uint32_t> foundTypes;

		auto callback = [&foundValues, &foundTypes](const std::wstring &valueName, uint32_t valueType)
		{
			foundValues.emplace(valueName);
			foundTypes.emplace(valueType);

			// Continue enumeration.
			return true;
		};

		// Enumerate registry values
		regkey->enumerateValues(callback);

		Assert::AreEqual(
			foundValues,
			values,
			L"Set of found values should equal set of created values"
		);

		// Found register values should have REG_SZ type
		// FIXME
		Assert::IsTrue(std::all_of(foundTypes.begin(), foundTypes.end(), [](const auto &type) {
			return type == REG_SZ;
		}), L"Found register values should have REG_SZ type");
	}
};

}
