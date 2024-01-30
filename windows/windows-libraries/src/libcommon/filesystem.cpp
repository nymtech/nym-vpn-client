#include "stdafx.h"
#include "filesystem.h"
#include "string.h"
#include "error.h"

namespace common::fs
{

void Mkdir(const std::wstring &path)
{
	if (path.empty())
	{
		return;
	}

	auto dirs = common::string::Tokenize(path, L"/\\");

	//
	// Implicit path so there is no work to be performed.
	//
	if (0 == dirs.size())
	{
		return;
	}

	//
	// Only the volume is specified so ignore this request.
	// TODO: It would be more correct to verify whether the volume exists.
	//
	if (1 == dirs.size())
	{
		return;
	}

	std::wstring target = L"\\\\?\\";

	auto it = dirs.begin();

	target.append(*it++).push_back(L'\\');

	DWORD lastError = ERROR_SUCCESS;

	do
	{
		target.append(*it++).push_back(L'\\');

		//
		// The first few levels can fail with ERROR_ACCESS_DENIED or
		// ERROR_ALREADY_EXISTS but we keep going and check the status on the final node.
		//
		lastError = (CreateDirectoryW(target.c_str(), nullptr) ? ERROR_SUCCESS : GetLastError());
	}
	while (it != dirs.end());

	if (ERROR_SUCCESS != lastError && ERROR_ALREADY_EXISTS != lastError)
	{
		auto msg = std::string("Failed to create directory: ").append(common::string::ToAnsi(target));

		THROW_ERROR(msg.c_str());
	}
}

std::wstring GetKnownFolderPath(REFKNOWNFOLDERID folderId, DWORD flags, HANDLE userToken)
{
	PWSTR folder = nullptr;

	const auto status = SHGetKnownFolderPath(folderId, flags, userToken, &folder);

	if (S_OK == status)
	{
		std::wstring result(folder);

		CoTaskMemFree(folder);

		return result;
	}

	CoTaskMemFree(folder);

	THROW_ERROR("Failed to retrieve \"known folder\" path");
}

ScopedNativeFileSystem::ScopedNativeFileSystem()
{
	if (FALSE == Wow64DisableWow64FsRedirection(&m_context))
	{
		THROW_WINDOWS_ERROR(GetLastError(), "Disable file system redirection");
	}
}

ScopedNativeFileSystem::~ScopedNativeFileSystem()
{
	if (FALSE == Wow64RevertWow64FsRedirection(m_context))
	{
		THROW_WINDOWS_ERROR(GetLastError(), "Revert file system redirection");
	}
}

}
