// Copyright 2016-2026 Mullvad VPN AB. All Rights Reserved.
// Copyright 2026 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

#include "st2.h"
#include <iostream>
#include <ws2tcpip.h>
#include "../util.h"
#include "../runtimesettings.h"
#include "../sockutil.h"
#include <libcommon/memory.h>

constexpr auto SocketRecvTimeoutValue = std::chrono::milliseconds(2000);

bool TestCaseSt2(const std::vector<std::wstring> &arguments)
{
	std::wcout << L"Launching split tunnel test case 2" << std::endl;
	std::wcout << L"Evaluate whether existing connections are blocked when an app becomes excluded" << std::endl;
	std::wcout << L"===" << std::endl;

	ArgumentContext argsContext(arguments);

	const auto tcp = ProtoArgumentTcp(argsContext.nextOrDefault(L"tcp"));

	argsContext.assertExhausted();

	PromptActivateVpn();

	std::wcout << "Creating socket and binding to tunnel IP" << std::endl;

	auto tunnelSocket = CreateBindSocket(RuntimeSettings::Instance().tunnelIp(), 0, tcp);

	common::memory::ScopeDestructor sd;

	sd += [&tunnelSocket]
	{
		ShutdownSocket(tunnelSocket);
	};

	if (!tcp)
	{
		SetSocketRecvTimeout(tunnelSocket, SocketRecvTimeoutValue);
	}

	std::wcout << L"Connecting to tcpbin server" << std::endl;

	ConnectSocket
	(
		tunnelSocket,
		RuntimeSettings::Instance().tcpbinServerIp(),
		tcp ? RuntimeSettings::Instance().tcpbinEchoPort() : RuntimeSettings::Instance().tcpbinEchoPortUdp()
	);

	std::wcout << L"Communicating with echo service to establish connectivity" << std::endl;

	SendRecvValidateEcho(tunnelSocket, { 'h', 'e', 'y', 'n', 'o', 'w' });

	std::wcout << L"Querying bind to verify correct interface is used" << std::endl;

	ValidateBind(tunnelSocket, RuntimeSettings::Instance().tunnelIp());

	PromptActivateSplitTunnel();

	std::wcout << L"Testing comms on LAN interface" << std::endl;

	auto lanSocket = CreateBindSocket(RuntimeSettings::Instance().lanIp(), 0, tcp);

	sd += [&lanSocket]
	{
		ShutdownSocket(lanSocket);
	};

	ConnectSocket
	(
		lanSocket,
		RuntimeSettings::Instance().tcpbinServerIp(),
		tcp ? RuntimeSettings::Instance().tcpbinEchoPort() : RuntimeSettings::Instance().tcpbinEchoPortUdp()
	);

	SendRecvValidateEcho(lanSocket, { 'h', 'e', 'y', 'n', 'o', 'w' });

	std::wcout << L"Sending and receiving to validate blocking policies" << std::endl;

	try
	{
		SendRecvValidateEcho(tunnelSocket, { 'b', 'l', 'o', 'c', 'k', 'e', 'd' });
	}
	catch (const std::exception &err)
	{
		std::cout << "Sending and receiving failed with message: " << err.what() << std::endl;
		std::wcout << L"Assuming firewall filters are blocking comms" << std::endl;

		return true;
	}

	std::wcout << L"Traffic leak!" << std::endl;

	return false;
}
