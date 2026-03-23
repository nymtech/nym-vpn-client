// Copyright 2016-2026 Mullvad VPN AB. All Rights Reserved.
// Copyright 2026 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

#define _WINSOCK_DEPRECATED_NO_WARNINGS 1

#include "st1.h"
#include <iostream>
#include <optional>
#include <ws2tcpip.h>	// include before windows.h
#include <libcommon/security.h>
#include <libcommon/error.h>
#include <libcommon/string.h>
#include "../util.h"
#include "../runtimesettings.h"
#include "../sockutil.h"
#include "../leaktest.h"

namespace
{

std::vector<uint8_t> GenerateEchoPayload()
{
	std::stringstream ss;

	ss << GetTickCount64();

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

void EvaluateSplitting(std::optional<IN_ADDR> bindAddr, const IN_ADDR &expectedActualBind, bool tcp)
{
	SOCKET s;
	
	if (bindAddr.has_value())
	{
		std::wcout << L"Creating socket and explicitly binding it" << std::endl;

		s = CreateBindSocket(bindAddr.value(), 0, tcp);
	}
	else
	{
		std::wcout << L"Creating socket and leaving it unbound" << std::endl;

		s = CreateSocket(tcp);
	}
	
	common::memory::ScopeDestructor sd;

	sd += [&s]
	{
		ShutdownSocket(s);
	};

	std::wcout << L"Connecting to tcpbin server" << std::endl;

	ConnectSocket
	(
		s,
		RuntimeSettings::Instance().tcpbinServerIp(),
		tcp ? RuntimeSettings::Instance().tcpbinEchoPort() : RuntimeSettings::Instance().tcpbinEchoPortUdp()
	);

	std::wcout << L"Communicating with echo service to establish connectivity" << std::endl;

	SendRecvValidateEcho(s, GenerateEchoPayload());

	std::wcout << L"Querying bind to verify correct interface is used" << std::endl;

	ValidateBind(s, expectedActualBind);
}

bool RunSubtest(std::optional<IN_ADDR> bindAddr, const IN_ADDR &expectedActualBind, bool tcp)
{
	try
	{
		EvaluateSplitting(bindAddr, expectedActualBind, tcp);

		return true;
	}
	catch (const std::exception &err)
	{
		std::wcout  << "EXCEPTION: " << err.what() << std::endl;

		return false;
	}
	catch (...)
	{
		std::cerr << "EXCEPTION: Unknown error" << std::endl;

		return false;
	}
}

} // anonymous namespace

bool TestCaseSt1(const std::vector<std::wstring> &arguments)
{
	std::wcout << L"Launching split tunnel test case 1" << std::endl;
	std::wcout << L"Evaluate whether different kinds of binds are correctly handled" << std::endl;
	std::wcout << L"===" << std::endl;

	ArgumentContext argsContext(arguments);

	const auto tcp = ProtoArgumentTcp(argsContext.nextOrDefault(L"tcp"));

	argsContext.assertExhausted();

	PromptActivateVpnSplitTunnel();

	//
	// There are three relevant tests:
	//
	// (localhost binds are tested separately)
	//
	// 1: Bind to tunnel and validate that bind is redirected to lan interface && that comms work
	// 2: Bind to lan interface and validate that bind is successful && that comms work
	// 3: Do not bind before connecting. Validate that bind is directed to lan interface && that comms work
	//

	std::wcout << L">> Testing explicit bind to tunnel interface" << std::endl;

	const auto subtest1 = RunSubtest
	(
		std::make_optional<>(RuntimeSettings::Instance().tunnelIp()),
		RuntimeSettings::Instance().lanIp(),
		tcp
	);

	std::wcout << L">> Testing explicit bind to LAN interface" << std::endl;

	const auto subtest2 = RunSubtest
	(
		std::make_optional<>(RuntimeSettings::Instance().lanIp()),
		RuntimeSettings::Instance().lanIp(),
		tcp
	);

	std::wcout << L">> Testing implicit bind" << std::endl;

	const auto subtest3 = RunSubtest
	(
		std::nullopt,
		RuntimeSettings::Instance().lanIp(),
		tcp
	);

	return subtest1 && subtest2 && subtest3;
}
