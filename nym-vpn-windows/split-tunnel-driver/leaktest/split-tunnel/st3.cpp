// Copyright 2016-2026 Mullvad VPN AB. All Rights Reserved.
// Copyright 2026 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

#include "st3.h"
#include <iostream>
#include <ws2tcpip.h>
#include "../util.h"
#include "../runtimesettings.h"
#include "../sockutil.h"
#include <libcommon/memory.h>

constexpr auto SocketRecvTimeoutValue = std::chrono::milliseconds(2000);

bool TestCaseSt3(const std::vector<std::wstring> &arguments)
{
	std::wcout << L"Launching split tunnel test case 3" << std::endl;
	std::wcout << L"Evaluate whether excluded connections are blocked when an app stops being excluded" << std::endl;
	std::wcout << L"===" << std::endl;

	ArgumentContext argsContext(arguments);

	const auto tcp = ProtoArgumentTcp(argsContext.nextOrDefault(L"tcp"));

	argsContext.assertExhausted();

	PromptActivateVpnSplitTunnel();

	std::wcout << "Creating socket and leaving it unbound" << std::endl;

	auto lanSocket = CreateSocket(tcp);

	common::memory::ScopeDestructor sd;

	sd += [&lanSocket]
	{
		ShutdownSocket(lanSocket);
	};

	if (!tcp)
	{
		SetSocketRecvTimeout(lanSocket, SocketRecvTimeoutValue);
	}

	std::wcout << L"Connecting to tcpbin server" << std::endl;

	//
	// Connecting will select the tunnel interface because of best metric,
	// but exclusion logic should redirect the bind.
	//

	ConnectSocket
	(
		lanSocket,
		RuntimeSettings::Instance().tcpbinServerIp(),
		tcp ? RuntimeSettings::Instance().tcpbinEchoPort() : RuntimeSettings::Instance().tcpbinEchoPortUdp()
	);

	std::wcout << L"Communicating with echo service to establish connectivity" << std::endl;

	SendRecvValidateEcho(lanSocket, { 'h', 'e', 'y', 'n', 'o', 'w' });

	std::wcout << L"Querying bind to verify correct interface is used" << std::endl;

	ValidateBind(lanSocket, RuntimeSettings::Instance().lanIp());

	PromptDisableSplitTunnel();

	std::wcout << L"Sending and receiving to validate blocking policies" << std::endl;

	try
	{
		SendRecvValidateEcho(lanSocket, { 'b', 'l', 'o', 'c', 'k', 'e', 'd' });
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
