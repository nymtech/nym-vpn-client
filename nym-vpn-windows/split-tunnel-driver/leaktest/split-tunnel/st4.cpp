// Copyright 2016-2026 Mullvad VPN AB. All Rights Reserved.
// Copyright 2026 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

#include "st4.h"
#include <iostream>
#include <ws2tcpip.h>
#include <windns.h>
#include "../util.h"
#include "../runtimesettings.h"
#include "../sockutil.h"
#include <libcommon/error.h>
#include <libcommon/process/applicationrunner.h>

using Runner = common::process::ApplicationRunner;

namespace
{

std::wstring GetPktmonPath()
{
	return L"c:\\windows\\system32\\pktmon.exe";
}

void PktmonCommand(const std::wstring &arguments, const char *errorMessage)
{
	auto command = Runner::StartWithoutConsole(GetPktmonPath(), arguments);

	DWORD status;

	command->join(status, INFINITE);

	if (status != 0)
	{
		THROW_ERROR(errorMessage);
	}
}

void PktmonRemoveFilters()
{
	PktmonCommand(L"filter remove", "Could not remove pktmon filters");
}

void PktmonAddFilter(const std::wstring &filterSpec)
{
	PktmonCommand(std::wstring(L"filter add ").append(filterSpec), "Could not add pktmon filter");
}

void PktmonStart()
{
	PktmonCommand(L"start", "Could not activate pktmon");
}

void PktmonStop()
{
	PktmonCommand(L"stop", "Could not stop pktmon");
}

std::string PktmonStopCaptureOutput()
{
	auto stopCommand = Runner::StartWithoutConsole(GetPktmonPath(), L"stop");

	DWORD status;

	stopCommand->join(status, INFINITE);

	if (status != 0)
	{
		THROW_ERROR("Could not stop pktmon");
	}

	std::string output;

	if (false == stopCommand->read(output, 1024, INFINITE))
	{
		THROW_ERROR("Stopped pktmon but could not capture output");
	}

	return output;
}

void SendDnsRequest()
{
	DNS_RECORDW *record = nullptr;

	for (auto i = 0; i < 3; ++i)
	{
		if (i != 0)
		{
			Sleep(1000);
		}

		std::wcout << L"Sending DNS query for A-record" << std::endl;

		const DWORD flags = DNS_QUERY_BYPASS_CACHE | DNS_QUERY_WIRE_ONLY;

		const auto status = DnsQuery_W(L"mullvad.net", DNS_TYPE_A, flags, nullptr, &record, nullptr);

		if (0 != status)
		{
			THROW_WINDOWS_ERROR(status, "Query DNS A-record");
		}

		const auto ip = IpToString(*reinterpret_cast<in_addr*>(&record->Data.A.IpAddress));

		std::wcout << L"Response" << std::endl;
		std::wcout << L"  name: " << record->pName << std::endl;
		std::wcout << L"  addr: " << ip << std::endl;

		DnsRecordListFree(record, DnsFreeRecordList);
	}
}

bool TestCaseSt4Inner(const std::vector<std::wstring> &arguments)
{
	std::wcout << L"Launching split tunnel test case 4" << std::endl;
	std::wcout << L"Evaluate whether DNS requests can be moved outside tunnel" << std::endl;
	std::wcout << L"===" << std::endl;

	ArgumentContext argsContext(arguments);

	argsContext.ensureExactArgumentCount(0);

	PromptActivateVpn();

	std::wcout << L"Getting pktmon ready" << std::endl;

	try
	{
		PktmonStop();
	}
	catch (...)
	{
	}

	//
	// Removing filters always succeeds with exit code 0.
	// Unless we're lacking an elevated token.
	//

	try
	{
		PktmonRemoveFilters();
	}
	catch (...)
	{
		THROW_ERROR("Re-launch test from elevated context");
	}

	std::wstringstream ss;

	ss << L"landns -d IPv4 --ip "
		<< IpToString(RuntimeSettings::Instance().lanIp())
		<< L" --port 53";

	PktmonAddFilter(ss.str());
	PktmonStart();

	PromptActivateSplitTunnel();

	SendDnsRequest();

	const auto output = PktmonStopCaptureOutput();

	const bool capturedNothing =
		(&output[0] == strstr(&output[0], "All counters are zero."));

	if (capturedNothing)
	{
		std::wcout << L"No DNS requests were captured on the LAN interface" << std::endl;

		return false;
	}

	//
	// There's nothing useful or identifying in the output.
	// One improvement would be to configure different DNS servers on the LAN interface
	// and the tunnel interface, and use a more specific filter in pktmon.
	//

	std::wcout << L"DNS requests successfully captured on LAN interface" << std::endl;

	return true;
}

} // anonymous namespace

bool TestCaseSt4(const std::vector<std::wstring> &arguments)
{
	auto cleanup = []()
	{
		try
		{
			PktmonStop();
		}
		catch (...)
		{
		}

		try
		{
			PktmonRemoveFilters();
		}
		catch (...)
		{
		}
	};

	try
	{
		const auto status = TestCaseSt4Inner(arguments);

		cleanup();

		return status;
	}
	catch (...)
	{
		cleanup();

		throw;
	}
}
