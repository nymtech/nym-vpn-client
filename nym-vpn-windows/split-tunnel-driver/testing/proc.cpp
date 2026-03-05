// Copyright 2016-2026 Mullvad VPN AB. All Rights Reserved.
// Copyright 2026 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

#include <vector>
#include <iterator>
#include <cstdint>
#include <windows.h>
#include <tlhelp32.h>
#include <libcommon/error.h>
#include <libcommon/memory.h>
#include <map>

#include "../nymvpn-split-tunnel/public.h"

#define PSAPI_VERSION 2
#include <psapi.h>

#include "proc.h"

using common::memory::UniqueHandle;

struct ProcessInfo
{
	DWORD ProcessId;
	DWORD ParentProcessId;
	FILETIME CreationTime;
	std::wstring DevicePath;
};

FILETIME GetProcessCreationTime(HANDLE processHandle)
{
	FILETIME creationTime, dummy;

	const auto status = GetProcessTimes(processHandle, &creationTime, &dummy, &dummy, &dummy);

	if (FALSE == status)
	{
		THROW_WINDOWS_ERROR(GetLastError(), "GetProcessTimes");
	}

	return creationTime;
}

std::wstring GetProcessDevicePath(HANDLE processHandle)
{
	size_t bufferSize = 512;
	std::vector<wchar_t> buffer;

	for (;;)
	{
		buffer.resize(bufferSize);

		const auto charsWritten = K32GetProcessImageFileNameW(processHandle,
			&buffer[0], static_cast<DWORD>(buffer.size()));

		if (0 == charsWritten)
		{
			if (ERROR_INSUFFICIENT_BUFFER == GetLastError())
			{
				bufferSize *= 2;
				continue;
			}

			THROW_WINDOWS_ERROR(GetLastError(), "K32GetProcessImageFileNameW");
		}

		//
		// K32GetProcessImageFileNameW writes a null terminator
		// but doesn't account for it in the return value.
		//

		return std::wstring(&buffer[0], &buffer[0] + charsWritten);
	}
}

//
// CompileProcessInfo()
//
// Returns a set including all processes in the system.
//
// The return value uses the vector container type since it's perceived
// the set will not be searched.
//
std::vector<ProcessInfo> CompileProcessInfo()
{
	auto snapshot = UniqueHandle(new HANDLE(
		(HANDLE)CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0)));

	if (INVALID_HANDLE_VALUE == *snapshot)
	{
		THROW_WINDOWS_ERROR(GetLastError(), "Snapshot processes");
	}

	PROCESSENTRY32W processEntry { .dwSize = sizeof(PROCESSENTRY32W) };

	if (FALSE == Process32First(*snapshot, &processEntry))
	{
		THROW_WINDOWS_ERROR(GetLastError(), "Initiate process enumeration");
	}

	std::map<DWORD, ProcessInfo> processes;

	//
	// Discover all processes.
	//

	do
	{
		auto handle = UniqueHandle(new HANDLE(OpenProcess(
			PROCESS_QUERY_LIMITED_INFORMATION, FALSE, processEntry.th32ProcessID)));

		if (NULL == *handle)
		{
			//THROW_WINDOWS_ERROR(GetLastError(), "Open process");
			continue;
		}

		ProcessInfo pi;

		pi.ProcessId = processEntry.th32ProcessID;
		pi.ParentProcessId = processEntry.th32ParentProcessID;

		try
		{
			pi.CreationTime = GetProcessCreationTime(*handle);
		}
		catch (...)
		{
			pi.CreationTime = { 0 };
		}

		try
		{
			pi.DevicePath = GetProcessDevicePath(*handle);
		}
		catch (...)
		{
			//
			// Including a process without a path might seem useless.
			// But it enables ancestor discovery.
			//

			pi.DevicePath = L"";
		}

		processes.insert(std::make_pair(pi.ProcessId, pi));
	}
	while (FALSE != Process32NextW(*snapshot, &processEntry));

	//
	// Find instances of PID recycling.
	//
	// This can be done by checking the creation time of the parent
	// process and discovering that the age of the claimed parent process
	// is lower than that of the child process.
	//

	for (auto& [pid, process] : processes)
	{
		auto parentIter = processes.find(process.ParentProcessId);

		if (parentIter != processes.end())
		{
			ULARGE_INTEGER parentTime { .LowPart = parentIter->second.CreationTime.dwLowDateTime,
				.HighPart = parentIter->second.CreationTime.dwHighDateTime };

			ULARGE_INTEGER processTime { .LowPart = process.CreationTime.dwLowDateTime,
				.HighPart = process.CreationTime.dwHighDateTime };

			if (0 != parentTime.QuadPart
				&& parentTime.QuadPart < processTime.QuadPart)
			{
				continue;
			}
		}

		process.ParentProcessId = 0;
	}

	//
	// Store process records into vector.
	//

	std::vector<ProcessInfo> output;

	output.reserve(processes.size());

	std::transform(processes.begin(), processes.end(), std::back_inserter(output),
		[](const std::map<DWORD, ProcessInfo>::value_type &entry)
		{
			return entry.second;
		});

	return output;
}

//
// MakeHandle()
//
// For some reason a PID is of type HANDLE in the kernel.
// Casting to HANDLE, which is a pointer type, requires some sourcery.
//
HANDLE MakeHandle(DWORD h)
{
	return reinterpret_cast<HANDLE>(static_cast<size_t>(h));
}

std::vector<uint8_t> PackageProcessInfo(const std::vector<ProcessInfo> &processes)
{
	if (processes.empty())
	{
		THROW_ERROR("Invalid set of processes (empty set)");
	}

	//
	// Determine required byte length for string buffer.
	//

	size_t stringBufferLength = 0;

	for (const auto &process : processes)
	{
		stringBufferLength += (process.DevicePath.size() * sizeof(wchar_t));
	}

	size_t bufferLength = sizeof(ST_PROCESS_DISCOVERY_HEADER)
		+ (sizeof(ST_PROCESS_DISCOVERY_ENTRY) * processes.size())
		+ stringBufferLength;

	std::vector<uint8_t> buffer(bufferLength);

	//
	// Create pointers to various buffer areas.
	//

	auto header = reinterpret_cast<ST_PROCESS_DISCOVERY_HEADER*>(&buffer[0]);
	auto entry = reinterpret_cast<ST_PROCESS_DISCOVERY_ENTRY*>(header + 1);
	auto stringBuffer = reinterpret_cast<uint8_t *>(entry + processes.size());

	//
	// Serialize into buffer.
	//

	SIZE_T stringOffset = 0;

	for (const auto &process : processes)
	{
		entry->ProcessId = MakeHandle(process.ProcessId);
		entry->ParentProcessId = MakeHandle(process.ParentProcessId);

		if (process.DevicePath.empty())
		{
			entry->ImageNameOffset = 0;
			entry->ImageNameLength = 0;
		}
		else
		{
			const auto imageNameLength = process.DevicePath.size() * sizeof(wchar_t);

			entry->ImageNameOffset = stringOffset;
			entry->ImageNameLength = static_cast<USHORT>(imageNameLength);

			RtlCopyMemory(stringBuffer + stringOffset, &process.DevicePath[0], imageNameLength);

			stringOffset += imageNameLength;
		}

		++entry;
	}

	//
	// Finalize header.
	//

	header->NumEntries = processes.size();
	header->TotalLength = bufferLength;

	return buffer;
}

std::vector<uint8_t> BuildRegisterProcessesPayload()
{
	return PackageProcessInfo(CompileProcessInfo());
}
