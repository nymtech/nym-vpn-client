#include "stdafx.h"
#include "registry.h"
#include "registrypath.h"
#include <libcommon/error.h>

namespace
{

REGSAM CreateAccessFlags(bool writeAccess, common::registry::RegistryView view)
{
	using namespace common::registry;

	REGSAM accessFlags = KEY_READ | (writeAccess ? KEY_WRITE : 0);

	switch (view)
	{
		case RegistryView::Force32:
		{
			accessFlags |= KEY_WOW64_32KEY;
			break;
		}
		case RegistryView::Force64:
		{
			accessFlags |= KEY_WOW64_64KEY;
			break;
		}
	};

	return accessFlags;
}

void DefaultMoveKey(const common::registry::RegistryPath &source, const common::registry::RegistryPath &destination)
{
	HKEY destHandle;

	auto status = RegCreateKeyExW(destination.key(), destination.subkey().c_str(), 0, nullptr, 0, KEY_ALL_ACCESS, nullptr, &destHandle, nullptr);

	if (ERROR_SUCCESS != status)
	{
		THROW_WINDOWS_ERROR(status, "Create destination key");
	}

	status = RegCopyTreeW(source.key(), source.subkey().c_str(), destHandle);

	RegCloseKey(destHandle);

	if (ERROR_SUCCESS != status)
	{
		THROW_WINDOWS_ERROR(status, "Copy registry key");
	}

	status = RegDeleteTreeW(source.key(), source.subkey().c_str());

	if (ERROR_SUCCESS != status)
	{
		THROW_WINDOWS_ERROR(status, "Clean up source tree");
	}

	status = RegDeleteKeyW(source.key(), source.subkey().c_str());

	if (ERROR_SUCCESS != status)
	{
		THROW_WINDOWS_ERROR(status, "Clean up source key");
	}
}

} // anonymous namespace

namespace common::registry
{

//static
std::unique_ptr<RegistryKey> Registry::CreateKey(HKEY key, const std::wstring &subkey, RegistryView view)
{
	const auto accessFlags = CreateAccessFlags(true, view);

	HKEY subkeyHandle;

	const auto status = RegCreateKeyExW(key, subkey.c_str(), 0, nullptr, 0, accessFlags, nullptr, &subkeyHandle, nullptr);

	if (ERROR_SUCCESS != status)
	{
		THROW_WINDOWS_ERROR(status, "Create registry key");
	}

	return std::make_unique<RegistryKey>(subkeyHandle);
}

//static
std::unique_ptr<RegistryKey> Registry::OpenKey(HKEY key, const std::wstring &subkey, bool writeAccess, RegistryView view)
{
	const auto accessFlags = CreateAccessFlags(writeAccess, view);

	HKEY subkeyHandle;

	const auto status = RegOpenKeyExW(key, subkey.c_str(), 0, accessFlags, &subkeyHandle);

	if (ERROR_SUCCESS != status)
	{
		THROW_WINDOWS_ERROR(status, "Open registry key");
	}

	return std::make_unique<RegistryKey>(subkeyHandle);
}

//static
void Registry::DeleteKey(HKEY key, const std::wstring &subkey, RegistryView view)
{
	LSTATUS status = ERROR_ACCESS_DENIED;

	if (RegistryView::Default == view)
	{
		status = RegDeleteKeyW(key, subkey.c_str());
	}
	else
	{
		const DWORD viewFlag = (RegistryView::Force32 == view ? KEY_WOW64_32KEY : KEY_WOW64_64KEY);

		status = RegDeleteKeyExW(key, subkey.c_str(), viewFlag, 0);
	}

	if (ERROR_SUCCESS != status
		&& ERROR_FILE_NOT_FOUND != status)
	{
		THROW_WINDOWS_ERROR(status, "Delete registry key");
	}
}

// static
void Registry::MoveKey
(
	HKEY sourceKey,
	const std::wstring &sourceSubkey,
	HKEY destinationKey,
	const std::wstring &destinationSubkey,
	RegistryView view
)
{
	RegistryPath source(sourceKey, sourceSubkey);
	RegistryPath destination(destinationKey, destinationSubkey);

	if (RegistryView::Default == view)
	{
		DefaultMoveKey(source, destination);
		return;
	}

	const REGSAM viewFlag = (RegistryView::Force32 == view ? KEY_WOW64_32KEY : KEY_WOW64_64KEY);

	auto releaseHkey = [](HKEY *key)
	{
		if (nullptr != *key)
		{
			RegCloseKey(*key);
		}

		delete key;
	};

	std::unique_ptr<HKEY, decltype(releaseHkey)>
		sourceHandle(new HKEY(nullptr), releaseHkey),
		destinationHandle(new HKEY(nullptr), releaseHkey);

 	auto status = RegOpenKeyExW(source.key(), source.subkey().c_str(), 0, KEY_ALL_ACCESS | viewFlag, sourceHandle.get());

	if (ERROR_SUCCESS != status)
	{
		THROW_WINDOWS_ERROR(status, "Open source key");
	}

	status = RegCreateKeyExW(destination.key(), destination.subkey().c_str(), 0, nullptr, 0, \
		KEY_ALL_ACCESS | viewFlag, nullptr, destinationHandle.get(), nullptr);

	if (ERROR_SUCCESS != status)
	{
		THROW_WINDOWS_ERROR(status, "Create destination key");
	}

	status = RegCopyTreeW(*sourceHandle, nullptr, *destinationHandle);

	if (ERROR_SUCCESS != status)
	{
		THROW_WINDOWS_ERROR(status, "Copy registry key");
	}

	status = RegDeleteTreeW(*sourceHandle, nullptr);

	if (ERROR_SUCCESS != status)
	{
		THROW_WINDOWS_ERROR(status, "Clean up source tree");
	}

	status = RegDeleteKeyExW(source.key(), source.subkey().c_str(), viewFlag, 0);

	if (ERROR_SUCCESS != status)
	{
		THROW_WINDOWS_ERROR(status, "Clean up source key");
	}
}

}
