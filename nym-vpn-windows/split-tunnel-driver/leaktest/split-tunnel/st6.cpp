// Copyright 2016-2026 Mullvad VPN AB. All Rights Reserved.
// Copyright 2026 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

#define _WINSOCK_DEPRECATED_NO_WARNINGS 1

#include "st6.h"
#include <iostream>
#include <ws2tcpip.h>	// include before windows.h
#include <libcommon/security.h>
#include <libcommon/error.h>
#include <libcommon/string.h>
#include "../util.h"
#include "../sockutil.h"
#include "../leaktest.h"

bool TestCaseSt6(const std::vector<std::wstring> &arguments)
{
	std::wcout << L"Launching split tunnel test case 6" << std::endl;
	std::wcout << L"Evaluate whether binds to localhost are correctly NOT being redirected" << std::endl;
	std::wcout << L"===" << std::endl;

	ArgumentContext argsContext(arguments);

	argsContext.ensureExactArgumentCount(0);

	PromptActivateVpnSplitTunnel();

	const auto serverAddr = std::wstring(L"127.0.0.1");
	const auto serverPort = std::wstring(L"5050");

	HANDLE serverProcess = NULL;
	HANDLE clientProcess = NULL;

	common::memory::ScopeDestructor sd;

   	sd += [&serverProcess, &clientProcess]()
	{
		if (serverProcess != NULL)
		{
			CloseHandle(serverProcess);
		}

		if (clientProcess != NULL)
		{
			CloseHandle(clientProcess);
		}
	};

	std::wcout << L"Launching server process" << std::endl;

	serverProcess = Fork(std::vector<std::wstring> {L"st6-server", serverAddr, serverPort});

	std::wcout << L"Waiting for VPN software to catch up" << std::endl;

	Sleep(1000 * 5);

	std::wcout << L"Launching unrelated client process" << std::endl;

	const auto childPath = ProcessBinaryCreateRandomCopy();

	sd += [&childPath]
	{
		DeleteFileW(childPath.c_str());
	};

	clientProcess = LaunchUnrelatedProcess
	(
		childPath,
		std::vector<std::wstring> {L"st6-client", serverAddr, serverPort},
		CREATE_NEW_CONSOLE
	);

	//
	// Wait for both processes to complete
	//

	HANDLE waitHandles[] =
	{
		serverProcess,
		clientProcess
	};

	const auto numWaitHandles = _countof(waitHandles);

	const auto waitStatus = WaitForMultipleObjects(numWaitHandles, waitHandles, TRUE, 1000 * 60);

	if (waitStatus < WAIT_OBJECT_0
		|| waitStatus > (WAIT_OBJECT_0 + numWaitHandles - 1))
	{
		THROW_ERROR("Failed waiting for tester processes");
	}

	//
	// Check return codes
	//

	DWORD serverProcessExitCode;
	DWORD clientProcessExitCode;

	if (FALSE == GetExitCodeProcess(serverProcess, &serverProcessExitCode)
		|| FALSE == GetExitCodeProcess(clientProcess, &clientProcessExitCode))
	{
		THROW_ERROR("Failed to acquire exit codes from tester processes");
	}

	return serverProcessExitCode == 0
		&& clientProcessExitCode == 0;
}

bool TestCaseSt6Server(const std::vector<std::wstring> &arguments)
{
	std::wcout << L"Split tunnel test case 6 - Server" << std::endl;
	std::wcout << L"I have PID: " << GetCurrentProcessId() << std::endl;

	SetPauseBeforeExit(true);

	ArgumentContext argsContext(arguments);

	argsContext.ensureExactArgumentCount(2);

	const auto serverAddr = argsContext.next();
	const auto serverPort = argsContext.next();

	std::wcout << L"Using server addr: " << serverAddr << L", port: " << serverPort << std::endl;
	std::wcout << L"Creating and binding server socket" << std::endl;

	//
	// Get server ready.
	//

	auto serverSocket = CreateBindSocket(serverAddr, common::string::LexicalCast<uint16_t>(serverPort));

	common::memory::ScopeDestructor sd;

	sd += [&serverSocket]
	{
		ShutdownSocket(serverSocket);
	};

	std::wcout << L"Calling listen() on socket" << std::endl;

	if (SOCKET_ERROR == listen(serverSocket, SOMAXCONN))
	{
		const auto err = std::string("Failed to listen on server socket: ")
			.append(FormatWsaError(WSAGetLastError()));

		THROW_ERROR(err.c_str());
	}

	//
	// Accept peer connection.
	//

	std::wcout << L"Calling accept() on server socket" << std::endl;

	sockaddr_in peer = { 0 };
	int peerSize = sizeof(peer);

	auto echoSocket = accept(serverSocket, (sockaddr*)&peer, &peerSize);

	if (INVALID_SOCKET == echoSocket)
	{
		const auto err = std::string("Failed to accept incoming connection: ")
			.append(FormatWsaError(WSAGetLastError()));

		THROW_ERROR(err.c_str());
	}

	sd += [&echoSocket]
	{
		ShutdownSocket(echoSocket);
	};

	if (peerSize != sizeof(peer))
	{
		THROW_ERROR("Invalid peer info returned");
	}

	//
	// Query server bind.
	//

	std::wcout << L"Retrieving bind details for server socket" << std::endl;

	auto local = QueryBind(serverSocket);

	std::wcout << L"Server endpoint: " << IpToString(local.sin_addr) << L":" << ntohs(local.sin_port) << std::endl;
	std::wcout << L"Peer: " << IpToString(peer.sin_addr) << L":" << ntohs(peer.sin_port) << std::endl;

	if (local.sin_addr != ParseIpv4(serverAddr))
	{
		THROW_ERROR("Unexpected server bind");
	}

	//
	// Enter echo loop.
	//

	std::wcout << L"Engaging in echo messaging" << std::endl;

	std::vector<uint8_t> buffer(1024);
	size_t index = 0;

	bool receivedSomething = false;

	for (;;)
	{
		if (index >= 1000)
		{
			index = 0;
		}

		const auto bytesReceived = recv(echoSocket, (char*)&buffer[index], 1, 0);

		if (bytesReceived != 1)
		{
			break;
		}

		receivedSomething = true;

		if (0x0a != buffer[index])
		{
			++index;
			continue;
		}

		buffer[++index] = 0;

		std::cout << (char*)&buffer[0];

		const auto bytesSent = send(echoSocket, (char*)&buffer[0], (int)index, 0);

		if (bytesSent != index)
		{
			THROW_ERROR("Failed to send echo response");
		}

		index = 0;

		if (nullptr != strstr((char*)&buffer[0], "exit"))
		{
			// Client requested to exit.
			break;
		}
	}

	return receivedSomething;
}

bool TestCaseSt6Client(const std::vector<std::wstring> &arguments)
{
	std::wcout << L"Split tunnel test case 6 - Client" << std::endl;
	std::wcout << L"I have PID: " << GetCurrentProcessId() << std::endl;

	SetPauseBeforeExit(true);

	ArgumentContext argsContext(arguments);

	argsContext.ensureExactArgumentCount(2);

	const auto serverAddr = argsContext.next();
	const auto serverPort = argsContext.next();

	std::wcout << L"Using server addr: " << serverAddr << L", port: " << serverPort << std::endl;
	std::wcout << L"Creating client socket" << std::endl;

	auto echoSocket = CreateSocket();

	common::memory::ScopeDestructor sd;

	sd += [&echoSocket]
	{
		ShutdownSocket(echoSocket);
	};

	//
	// Connect.
	//
	// This will momentarily bind the socket to "0.0.0.0".
	// The initial bind event is passed through WFP bind redirect callouts.
	//
	// The bind will be corrected shortly thereafter to reflect
	// which interface was actually bound to.
	//
	// The corrected bind is not reported to WFP callouts.
	//

	std::wcout << L"Connecting socket" << std::endl;

	ConnectSocket(echoSocket, serverAddr, common::string::LexicalCast<uint16_t>(serverPort));

	//
	// Query bind after connecting.
	//
	// This is a little backwards, the comparison only works because we know the server
	// is running on localhost.
	//

	std::wcout << L"Retrieving bind details for socket" << std::endl;

	auto bindInfo = QueryBind(echoSocket);

	std::wcout << L"Bind details: " << IpToString(bindInfo.sin_addr) << L":" << ntohs(bindInfo.sin_port) << std::endl;

	if (bindInfo.sin_addr != ParseIpv4(serverAddr))
	{
		THROW_ERROR("Unexpected socket bind");
	}

	//
	// Ensure socket can send/receive.
	//

	std::wcout << L"Verifying connection" << std::endl;

	std::vector<uint8_t> out = { 'm', 'e', 'e', 'p', '\x0a' };

	auto in = SendRecvSocket(echoSocket, out);

	if (in.size() != out.size()
		|| 0 != memcmp(&in[0], &out[0], in.size()))
	{
		THROW_ERROR("Invalid echo response");
	}

	std::wcout << L"Echo response OK" << std::endl;

	//
	// Query bind after comms.
	//

	std::wcout << L"Retrieving bind details for socket" << std::endl;

	bindInfo = QueryBind(echoSocket);

	std::wcout << L"Bind details: " << IpToString(bindInfo.sin_addr) << L":" << ntohs(bindInfo.sin_port) << std::endl;

	if (bindInfo.sin_addr != ParseIpv4(serverAddr))
	{
		THROW_ERROR("Unexpected socket bind");
	}

	return true;
}
