// Copyright 2016-2026 Mullvad VPN AB. All Rights Reserved.
// Copyright 2026 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

#define _WINSOCK_DEPRECATED_NO_WARNINGS 1

#include "st7.h"
#include <iostream>
#include <ws2tcpip.h>
#include <libcommon/security.h>
#include <libcommon/error.h>
#include <libcommon/string.h>
#include "../runtimesettings.h"
#include "../util.h"
#include "../sockutil.h"
#include "../leaktest.h"

// Signalled by child to indicate readiness.
const wchar_t *ReadyEventName = L"ST7-CHILD-READY-EVENT";

// Signalled by parent to command child into action.
const wchar_t *CommenceEventName = L"ST7-CHILD-COMMENCE-EVENT";

enum class ChildStatus
{
	// If child process fails to override exit code before completing.
	// This "can't" happen.
	GeneralSuccess = 0,

	// If child process throws before able to complete.
	GeneralFailure = 1,

	// Explicit bind to tunnel was successful.
	BindUnaffected = 2,

	// Explicit bind to tunnel was redirected to LAN interface.
	BindRedirected = 3
};

bool TestCaseSt7(const std::vector<std::wstring> &arguments)
{
	std::wcout << L"Launching split tunnel test case 7" << std::endl;
	std::wcout << L"Evaluate whether existing child processes become excluded with their parent" << std::endl;
	std::wcout << L"===" << std::endl;

	ArgumentContext argsContext(arguments);

	argsContext.ensureExactArgumentCount(0);

	PromptActivateVpn();

	//
	// The child process needs to wait for an event to become signalled.
	// So it makes the connection attempt at the right moment.
	//

	std::wcout << L"Creating synchronization events" << std::endl;

	HANDLE readyEvent = CreateEventW(nullptr, TRUE, FALSE, ReadyEventName);

	if (NULL == readyEvent || GetLastError() == ERROR_ALREADY_EXISTS)
	{
		THROW_ERROR("Could not create event object");
	}

	common::memory::ScopeDestructor sd;

	sd += [readyEvent]
	{
		CloseHandle(readyEvent);
	};

	HANDLE commenceEvent = CreateEventW(nullptr, TRUE, FALSE, CommenceEventName);

	if (NULL == commenceEvent || GetLastError() == ERROR_ALREADY_EXISTS)
	{
		THROW_ERROR("Could not create event object");
	}

	sd += [commenceEvent]
	{
		CloseHandle(commenceEvent);
	};

	std::wcout << L"Creating child process" << std::endl;

	const auto childPath = ProcessBinaryCreateRandomCopy();

	sd += [&childPath]
	{
		DeleteFileW(childPath.c_str());
	};

	auto childProcess = LaunchProcess
	(
		childPath,
		{ L"st7-child", RuntimeSettings::GetSettingsFilePath() },
		CREATE_NEW_CONSOLE
	);

	sd += [childProcess]
	{
		// This will fail if the child process was successful.
		TerminateProcess(childProcess, 0x1337);

		CloseHandle(childProcess);
	};

	std::wcout << L"Waiting for child process to become ready" << std::endl;

	WaitForSingleObject(readyEvent, INFINITE);

	PromptActivateSplitTunnel();

	std::wcout << L"Commanding child process to make a connection" << std::endl;

	SetEvent(commenceEvent);

	std::wcout << L"Waiting for child process to finish" << std::endl;

	WaitForSingleObject(childProcess, INFINITE);

	DWORD exitCode;

	auto status = GetExitCodeProcess(childProcess, &exitCode);

	if (FALSE == status)
	{
		THROW_ERROR("Could not determine child process exit code");
	}

	switch ((ChildStatus)exitCode)
	{
		case ChildStatus::BindUnaffected:
		{
			std::wcout << L"Socket binds in child process are NOT redirected" << std::endl;

			return false;
		}
		case ChildStatus::BindRedirected:
		{
			std::wcout << L"Socket binds in child process are being redirected" << std::endl;

			return true;
		}
	};

	std::wcout << L"Unexpected child process exit code" << std::endl;

	return false;
}

bool TestCaseSt7Child(const std::vector<std::wstring> &arguments)
{
	std::wcout << L"Launching split tunnel test case 7 - CHILD" << std::endl;
	std::wcout << L"===" << std::endl;

	SetPauseBeforeExit(true);

	ArgumentContext argsContext(arguments);

	auto settingsFilePath = argsContext.next();

	RuntimeSettings::OverrideSettingsFilePath(settingsFilePath);

	argsContext.assertExhausted();

	std::wcout << L"Opening event objects" << std::endl;

	constexpr DWORD AccessRights = STANDARD_RIGHTS_READ | SYNCHRONIZE | EVENT_MODIFY_STATE;

	auto readyEvent = OpenEventW(AccessRights, FALSE, ReadyEventName);

	if (NULL == readyEvent)
	{
		THROW_ERROR("Could not open event object");
	}

	common::memory::ScopeDestructor sd;

	sd += [readyEvent]
	{
		CloseHandle(readyEvent);
	};

	auto commenceEvent = OpenEventW(AccessRights, FALSE, CommenceEventName);

	if (NULL == commenceEvent)
	{
		THROW_ERROR("Could not open event object");
	}

	sd += [commenceEvent]
	{
		CloseHandle(commenceEvent);
	};

	std::wcout << L"Signalling to parent that child is ready" << std::endl;

	SetEvent(readyEvent);

	std::wcout << L"Waiting for parent" << std::endl;

	WaitForSingleObject(commenceEvent, INFINITE);

	std::wcout << L"Creating socket and binding to tunnel interface" << std::endl;

	auto socket = CreateBindSocket(RuntimeSettings::Instance().tunnelIp());

	sd += [&socket]
	{
		ShutdownSocket(socket);
	};

	std::wcout << L"Connecting to tcpbin echo service" << std::endl;

	ConnectSocket
	(
		socket,
		RuntimeSettings::Instance().tcpbinServerIp(),
		RuntimeSettings::Instance().tcpbinEchoPort()
	);

	std::wcout << L"Communicating with echo service to establish connectivity" << std::endl;

	SendRecvValidateEcho(socket, { 'h', 'e', 'y', 'n', 'o', 'w' });

	std::wcout << L"Querying bind to determine which interface is used" << std::endl;

	const auto actualBind = QueryBind(socket);

	if (actualBind.sin_addr == RuntimeSettings::Instance().tunnelIp())
	{
		std::wcout << L"Bound to tunnel interface" << std::endl;

		SetProcessExitCode(static_cast<int>(ChildStatus::BindUnaffected));

		return true;
	}
	else if (actualBind.sin_addr == RuntimeSettings::Instance().lanIp())
	{
		std::wcout << L"Bound to LAN interface" << std::endl;

		SetProcessExitCode(static_cast<int>(ChildStatus::BindRedirected));

		return true;
	}

	std::wcout << L"Failed to match bind to known interface" << std::endl;

	return false;
}
