// Copyright 2016-2026 Mullvad VPN AB. All Rights Reserved.
// Copyright 2026 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

#include <ws2tcpip.h>	// include before windows.h
#include "general/gen1.h"
#include "split-tunnel/st1.h"
#include "split-tunnel/st2.h"
#include "split-tunnel/st3.h"
#include "split-tunnel/st4.h"
#include "split-tunnel/st5.h"
#include "split-tunnel/st6.h"
#include "split-tunnel/st7.h"
#include "util.h"
#include "sockutil.h"
#include <libcommon/error.h>
#include <libcommon/string.h>
#include <iostream>
#include <vector>
#include <string>
#include <sstream>
#include <functional>
#include <windows.h>

namespace
{

bool g_PauseBeforeExit = false;

int g_ProcessExitCode = 0;
bool g_UseProcessExitCode = false;

} // anonymous namespace

void SetPauseBeforeExit(bool pause)
{
	g_PauseBeforeExit = pause;
}

void MaybePauseBeforeExit()
{
	if (g_PauseBeforeExit)
	{
		std::wcout << L"Press a key to continue..." << std::endl;
		_getwch();
	}
}

void SetProcessExitCode(int exitCode)
{
	g_ProcessExitCode = exitCode;
	g_UseProcessExitCode = true;
}

bool innerMain(int argc, wchar_t *argv[])
{
	if (argc < 2)
	{
		THROW_ERROR("Test ID not specified");
	}

	const std::wstring testId = argv[1];

	std::vector<std::wstring> arguments;

	for (size_t argumentIndex = 2; argumentIndex < argc; ++argumentIndex)
	{
		arguments.emplace_back(argv[argumentIndex]);
	}

	//
	// Declare all test cases.
	//

	struct TestCase
	{
		std::wstring id;
		std::function<bool(const std::vector<std::wstring> &)> handler;
	};

	std::vector<TestCase> tests =
	{
		{ L"gen1", TestCaseGen1},
		{ L"st1", TestCaseSt1 },
		{ L"st2", TestCaseSt2 },
		{ L"st3", TestCaseSt3 },
		{ L"st4", TestCaseSt4 },
		{ L"st5", TestCaseSt5 },
		{ L"st5-child", TestCaseSt5Child },
		{ L"st6", TestCaseSt6 },
		{ L"st6-server", TestCaseSt6Server },
		{ L"st6-client", TestCaseSt6Client },
		{ L"st7", TestCaseSt7 },
		{ L"st7-child", TestCaseSt7Child },
	};

	//
	// Find and invoke matching handler.
	//

	for (const auto &candidate : tests)
	{
		if (0 != _wcsicmp(testId.c_str(), candidate.id.c_str()))
		{
			continue;
		}

		return candidate.handler(arguments);
	}

	//
	// Invalid test id specified on command line.
	//

	std::stringstream ss;

	ss << "Invalid test id: " << common::string::ToAnsi(testId);

	THROW_ERROR(ss.str().c_str());
}

int wmain(int argc, wchar_t *argv[])
{
	WSADATA winSockData;

	if (0 != WSAStartup(MAKEWORD(2, 2), &winSockData))
	{
		std::cerr << "Failed to initialize winsock: " << FormatWsaError(WSAGetLastError()) << std::endl;

		return 1;
	}

	if (LOBYTE(winSockData.wVersion) != 2 || HIBYTE(winSockData.wVersion) != 2)
	{
		WSACleanup();

		std::cerr << "Could not find/load winsock 2.2 implementation" << std::endl;

		return 1;
	}

	auto successful = false;

	try
	{
		successful = innerMain(argc, argv);
	}
	catch (const std::exception &err)
	{
		std::cerr << "EXCEPTION: " << err.what() << std::endl;
	}
	catch (...)
	{
		std::cerr << "EXCEPTION: Unknown error" << std::endl;
	}

	if (successful)
	{
		PrintGreen(L"--> PASS <--");
	}
	else
	{
		PrintRed(L"!!! FAIL !!!");
	}

	MaybePauseBeforeExit();

	if (successful)
	{
		return (g_UseProcessExitCode ? g_ProcessExitCode : 0);
	}

	return 1;
}
