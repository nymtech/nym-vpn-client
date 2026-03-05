// Copyright 2016-2026 Mullvad VPN AB. All Rights Reserved.
// Copyright 2026 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

#include "st5.h"
#include <iostream>
#include <ws2tcpip.h>
#include "../util.h"
#include "../runtimesettings.h"
#include "../sockutil.h"
#include "../leaktest.h"
#include <libcommon/memory.h>
#include <chrono>

namespace
{

class WaitAssistant
{
public:

	WaitAssistant(DWORD maxWaitTimeMs)
		: m_maxWaitTime(maxWaitTimeMs)
	{
	}

	//
	// Returns max time in ms that can be waited for.
	// Or 0 if the max waiting time has already been reached.
	//
	DWORD getWaitTime() const
	{
		const auto now = std::chrono::steady_clock::now();

		const auto elapsed = std::chrono::duration_cast<std::chrono::milliseconds>(now - startTime);

		return (elapsed > m_maxWaitTime ? 0 : static_cast<DWORD>((m_maxWaitTime - elapsed).count()));
	}

private:

	const std::chrono::milliseconds m_maxWaitTime;

	using time_point = std::chrono::time_point<std::chrono::steady_clock>;

	const time_point startTime = std::chrono::steady_clock::now();
};

HANDLE
LaunchChild
(
	const std::filesystem::path &path,
	const std::filesystem::path &settingsFilePath,
	const std::wstring &tunnelIp,
	const std::wstring &lanIp
)
{
	const auto quotedSettingsFile = std::wstring(L"\"").append(settingsFilePath).append(L"\"");

	return LaunchProcess
	(
		path,
		std::vector<std::wstring> {L"st5-child", quotedSettingsFile, tunnelIp, lanIp},
		CREATE_NO_WINDOW
	);
}

enum class ChildExitCode
{
	// If child process fails to override exit code before completing.
	// This "can't" happen.
	GeneralSuccess = 0,

	// If child process throws before able to complete.
	GeneralFailure = 1,

	// Bind was successful and was NOT redirected.
	BoundTunnel = 2,

	// Bind was successful and was redirected.
	BoundLan = 3
};

std::vector<uint8_t> GenerateChildPayload()
{
	std::stringstream ss;

	ss << "st-5-child:" << GetCurrentProcessId();

	std::vector<uint8_t> payload;

	//
	// stringstream returns a copy, not a reference.
	//
	const auto source = ss.str();

	payload.reserve(source.size());

	std::transform(source.begin(), source.end(), std::back_inserter(payload), [](char c)
	{
		return static_cast<uint8_t>(c);
	});

	return payload;
}

} // anonymous namespace

bool TestCaseSt5(const std::vector<std::wstring> &arguments)
{
	std::wcout << L"Launching split tunnel test case 5" << std::endl;
	std::wcout << L"Evaluate whether child processes are automatically and atomically handled" << std::endl;
	std::wcout << L"===" << std::endl;

	ArgumentContext argsContext(arguments);

	argsContext.ensureExactArgumentCount(0);

	PromptActivateVpnSplitTunnel();

	const size_t NUM_PROCESSES = 20;

	std::wcout << L"Starting " << NUM_PROCESSES << L" processes to test child process association" << std::endl;
	std::wcout << L"Each child will attempt to bind to the tunnel interface" << std::endl;

	struct ScoreCard
	{
		uint64_t miscFailure;
		uint64_t tunnelBind;
		uint64_t lanBind;
	}
	scoreCard = {0,0,0};

	const auto settingsFilePath = RuntimeSettings::GetSettingsFilePath();
	const auto tunnelIp = IpToString(RuntimeSettings::Instance().tunnelIp());
	const auto lanIp = IpToString(RuntimeSettings::Instance().lanIp());

	//
	// Create unique binaries, one for each child.
	// This way the VPN can't use the path to determine association.
	//

	std::vector<std::filesystem::path> childPaths;

	common::memory::ScopeDestructor sd;

	sd += [&childPaths]
	{
		for(const auto &path : childPaths)
		{
			DeleteFileW(path.c_str());
		}
	};

	childPaths.reserve(NUM_PROCESSES);

	for (auto i = 0; i < NUM_PROCESSES; ++i)
	{
		childPaths.emplace_back(ProcessBinaryCreateRandomCopy());
	}

	//
	// Launch all processes in quick succession, without waiting for each one to complete its work.
	// This should hopefully create some CPU load and work for the scheduler.
	//

	std::vector<HANDLE> childProcesses;

	sd += [&childProcesses]()
	{
		for (auto process : childProcesses)
		{
			CloseHandle(process);
		}
	};

	childProcesses.reserve(NUM_PROCESSES);

	for (auto i = 0; i < NUM_PROCESSES; ++i)
	{
		try
		{
			childProcesses.emplace_back(LaunchChild(childPaths[i], settingsFilePath, tunnelIp, lanIp));
		}
		catch (...)
		{
			std::wcout << L"Failed to launch child process" << std::endl;
			++scoreCard.miscFailure;
		}
	}

	const size_t MAX_WAIT_TIME_MS = 1000 * 10;

	WaitAssistant waitAssistant(MAX_WAIT_TIME_MS);

	for (auto process : childProcesses)
	{
		auto status = WaitForSingleObject(process, waitAssistant.getWaitTime());

		if (WAIT_OBJECT_0 != status)
		{
			std::wcout << L"Child process did not complete in time" << std::endl;

			++scoreCard.miscFailure;
			continue;
		}

		DWORD exitCode;

		if (FALSE == GetExitCodeProcess(process, &exitCode))
		{
			std::wcout << L"Failed to read child process exit code" << std::endl;

			++scoreCard.miscFailure;
			continue;
		}

		switch ((ChildExitCode)exitCode)
		{
			case ChildExitCode::BoundLan:
			{
				++scoreCard.lanBind;
				break;
			}
			case ChildExitCode::BoundTunnel:
			{
				++scoreCard.tunnelBind;
				break;
			}
			default:
			{
				std::wcout << L"Unexpected child exit code" << std::endl;

				++scoreCard.miscFailure;
			}
		}
	}

	std::wcout << L"-----" << std::endl;
	std::wcout << L"Failed to start or report status: " << scoreCard.miscFailure << std::endl;
	std::wcout << L"Had their bind redirected: " << scoreCard.lanBind << std::endl;
	std::wcout << L"Bypassed the split tunnel functionality: " << scoreCard.tunnelBind << std::endl;
	std::wcout << L"-----" << std::endl;

	return NUM_PROCESSES == scoreCard.lanBind;
}

bool TestCaseSt5Child(const std::vector<std::wstring> &arguments)
{
	std::wcout << L"Split tunnel test case 5 - Child" << std::endl;
	std::wcout << L"I have PID: " << GetCurrentProcessId() << std::endl;

	ArgumentContext argsContext(arguments);

	argsContext.ensureExactArgumentCount(3);

	const auto settingsFilePath = std::filesystem::path(argsContext.next());
	const auto tunnelIp = ParseIpv4(argsContext.next());
	const auto lanIp = ParseIpv4(argsContext.next());

	RuntimeSettings::OverrideSettingsFilePath(settingsFilePath);

	std::wcout << "Creating socket and leaving it unbound" << std::endl;

	auto lanSocket = CreateSocket();

	common::memory::ScopeDestructor sd;

	sd += [&lanSocket]
	{
		ShutdownSocket(lanSocket);
	};

	std::wcout << L"Connecting to tcpbin server" << std::endl;

	//
	// Connecting will select the tunnel interface because of best metric,
	// but exclusion logic should redirect the bind.
	//

	ConnectSocket
	(
		lanSocket,
		RuntimeSettings::Instance().tcpbinServerIp(),
		RuntimeSettings::Instance().tcpbinEchoPort()
	);

	std::wcout << L"Communicating with echo service to establish connectivity" << std::endl;

	const auto payload = GenerateChildPayload();

	SendRecvValidateEcho(lanSocket, payload);

	std::wcout << L"Querying bind to determine which interface is used" << std::endl;

	const auto actualBind = QueryBind(lanSocket);

	if (actualBind.sin_addr == tunnelIp)
	{
		std::wcout << L"Bound to tunnel interface" << std::endl;

		SetProcessExitCode(static_cast<int>(ChildExitCode::BoundTunnel));

		return true;
	}
	else if (actualBind.sin_addr == lanIp)
	{
		std::wcout << L"Bound to LAN interface" << std::endl;

		SetProcessExitCode(static_cast<int>(ChildExitCode::BoundLan));

		return true;
	}

	std::wcout << L"Failed to match bind to known interface" << std::endl;

	return false;
}
