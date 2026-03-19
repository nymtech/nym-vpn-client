// Copyright 2016-2026 Mullvad VPN AB. All Rights Reserved.
// Copyright 2026 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

#include "gen1.h"
#include "../util.h"
#include "../sockutil.h"
#include "../runtimesettings.h"
#include <libcommon/memory.h>
#include <libcommon/error.h>
#include <iostream>

namespace
{

std::vector<uint8_t> GenerateEchoBuffer()
{
	return { 0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17 };
}

void LoopSendReceive(SOCKET lanSocket, size_t delay)
{
	WinsockOverlapped *sendContext;
	WinsockOverlapped *recvContext;

	common::memory::ScopeDestructor sd;

	sd += [&lanSocket, &sendContext, &recvContext]
	{
		//
		// This has to happen first so pending operations are cancelled.
		//
		ShutdownSocket(lanSocket);

		DeleteWinsockOverlapped(&sendContext);
		DeleteWinsockOverlapped(&recvContext);
	};

	sendContext = AllocateWinsockOverlapped();
	recvContext = AllocateWinsockOverlapped();

	for (;;)
	{
		if (!sendContext->pendingOperation)
		{
			AssignOverlappedBuffer(*sendContext, GenerateEchoBuffer());
			SendOverlappedSocket(lanSocket, *sendContext);
		}

		if (!recvContext->pendingOperation)
		{
			RecvOverlappedSocket(lanSocket, *recvContext, 1024);
		}

		Sleep(static_cast<DWORD>(delay));

		//
		// Check if overlapped ops have completed.
		// 

		const bool sendCompleted = PollOverlappedSend(lanSocket, *sendContext);
		const bool recvCompleted = PollOverlappedRecv(lanSocket, *recvContext);

		if (sendCompleted)
		{
			if (recvCompleted)
			{
				std::wcout << L'+';
			}
			else
			{
				std::wcout << L's';
			}
		}
		else if (recvCompleted)
		{
			std::wcout << L'r';
		}
	}
}

SOCKET CreateConnectSocket(bool tcp, bool verbose)
{
	if (verbose)
	{
		std::wcout << L"Creating socket and binding to LAN interface" << std::endl;
	}

	auto lanSocket = CreateBindOverlappedSocket(RuntimeSettings::Instance().lanIp(), 0, tcp);

	try
	{
		if (verbose)
		{
			std::wcout << L"Connecting to tcpbin echo service" << std::endl;
		}

		ConnectSocket
		(
			lanSocket,
			RuntimeSettings::Instance().tcpbinServerIp(),
			tcp ? RuntimeSettings::Instance().tcpbinEchoPort() : RuntimeSettings::Instance().tcpbinEchoPortUdp()
		);
	}
	catch (...)
	{
		ShutdownSocket(lanSocket);

		throw;
	}

	return lanSocket;
}

template<typename T>
T take(T &lhs, T rhs)
{
	T temp = lhs;
	lhs = rhs;
	return temp;
}

} // anonymous namespace

//
// Disable `warning C4702: unreachable code`
// The function mostly doesn't return.
//
#pragma warning(push)
#pragma warning(disable:4702)

bool TestCaseGen1(const std::vector<std::wstring> &arguments)
{
	std::wcout << L"Launching general test case 1" << std::endl;
	std::wcout << L"Evaluate whether VPN client state changes have momentary leaks" << std::endl;
	std::wcout << L"===" << std::endl;

	ArgumentContext argsContext(arguments);

	const auto tcp = ProtoArgumentTcp(argsContext.nextOrDefault(L"tcp"));
	const auto delay = common::string::LexicalCast<size_t>(argsContext.nextOrDefault(L"50"));

	argsContext.assertExhausted();

	auto lanSocket = CreateConnectSocket(tcp, true);

	PromptActivateVpn();

	std::wcout << L"You should interact with the VPN app to cause state changes" << std::endl;
	std::wcout << L"'s' is successfully sent data" << std::endl;
	std::wcout << L"'r' is successfully received data" << std::endl;
	std::wcout << L"'+' is a successful send+receive" << std::endl;
	std::wcout << L"'.' is a broken socket that's being reconnected" << std::endl;

	bool brokenSocket = false;

	for (;;)
	{
		try
		{
			if (brokenSocket)
			{
				Sleep(static_cast<DWORD>(delay));

				lanSocket = CreateConnectSocket(tcp, false);

				brokenSocket = false;
			}

			//
			// NOTE: Ownership of `lanSocket` is passed to LoopSendReceive().
			// The socket will already be closed if the function ever returns.
			//

			LoopSendReceive(take<>(lanSocket, SOCKET(INVALID_SOCKET)), delay);
		}
		catch (const common::error::WindowsException &err)
		{
			if (brokenSocket)
			{
				std::wcout << L'.';

				continue;
			}

			if (err.errorCode() == WSAECONNRESET
				|| err.errorCode() == WSAECONNABORTED)
			{
				std::wcout << L'.';

				brokenSocket = true;

				continue;
			}

			throw;
		}
	}

	return false; 	//NOSONAR
}
#pragma warning(pop)
