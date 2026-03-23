// Copyright 2016-2026 Mullvad VPN AB. All Rights Reserved.
// Copyright 2026 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

#include "sockutil.h"
#include "util.h"
#include <libcommon/error.h>
#include <libcommon/string.h>
#include <stdexcept>
#include <sstream>
#include <iomanip>
#include <cassert>

std::string FormatWsaError(int errorCode)
{
	std::stringstream ss;

	ss << "0x" << std::setw(8) << std::setfill('0') << std::hex << errorCode;

	return ss.str();
}

SOCKET CreateBindSocket(const IN_ADDR &ip, uint16_t port, bool tcp)
{
	auto s = CreateSocket(tcp);

	sockaddr_in endpoint = { 0 };

	endpoint.sin_family = AF_INET;
	endpoint.sin_port = htons(port);
	endpoint.sin_addr = ip;

	auto status = bind(s, (sockaddr*)&endpoint, sizeof(endpoint));

	if (SOCKET_ERROR == status)
	{
		const auto errorCode = WSAGetLastError();

		closesocket(s);

		const auto err = std::string("Failed to bind socket: ")
			.append(FormatWsaError(errorCode));

		THROW_ERROR(err.c_str());
	}

	return s;
}

SOCKET CreateBindSocket(const std::wstring &ip, uint16_t port, bool tcp)
{
	return CreateBindSocket(ParseIpv4(ip), port, tcp);
}

SOCKET CreateSocket(bool tcp)
{
	const auto [type, protocol] = tcp
		? std::make_pair<>(SOCK_STREAM, IPPROTO_TCP)
		: std::make_pair<>(SOCK_DGRAM, IPPROTO_UDP);

	auto s = socket(AF_INET, type, protocol);

	if (INVALID_SOCKET == s)
	{
		THROW_ERROR("Failed to create socket");
	}

	return s;
}

void ShutdownSocket(SOCKET &s)
{
	if (s != INVALID_SOCKET)
	{
		shutdown(s, SD_BOTH);
		closesocket(s);

		s = INVALID_SOCKET;
	}
}

void ConnectSocket(SOCKET s, const IN_ADDR &ip, uint16_t port)
{
	sockaddr_in peer = { 0 };

	peer.sin_family = AF_INET;
	peer.sin_port = htons(port);
	peer.sin_addr = ip;

	if (SOCKET_ERROR == connect(s, (sockaddr*)&peer, sizeof(peer)))
	{
		const auto lastError = WSAGetLastError();

		const auto err = std::string("Failed to connect socket: ")
			.append(FormatWsaError(lastError));

		THROW_WINDOWS_ERROR(lastError, err.c_str());
	}
}

void ConnectSocket(SOCKET s, const std::wstring &ip, uint16_t port)
{
	ConnectSocket(s, ParseIpv4(ip), port);
}

std::vector<uint8_t> SendRecvSocket(SOCKET s, const std::vector<uint8_t> &sendBuffer)
{
	auto status = send(s, (const char *)&sendBuffer[0], (int)sendBuffer.size(), 0);

	if (SOCKET_ERROR == status)
	{
		const auto err = std::string("Failed to send on socket: ")
			.append(FormatWsaError(WSAGetLastError()));

		THROW_ERROR(err.c_str());
	}

	if (status != sendBuffer.size())
	{
		std::stringstream ss;

		ss << "Failed to send() on socket. Sent " << status << " of " << sendBuffer.size() << " bytes";

		THROW_ERROR(ss.str().c_str());
	}

	std::vector<uint8_t> receiveBuffer(1024, 0);

	status = recv(s, (char *)&receiveBuffer[0], (int)receiveBuffer.size(), 0);

	if (SOCKET_ERROR == status)
	{
		const auto err = std::string("Failed to receive on socket: ")
			.append(FormatWsaError(WSAGetLastError()));

		THROW_ERROR(err.c_str());
	}

	receiveBuffer.resize(status);

	return receiveBuffer;
}

void SendRecvValidateEcho(SOCKET s, const std::vector<uint8_t> &sendBuffer)
{
	const auto receiveBuffer = SendRecvSocket(s, sendBuffer);

	if (receiveBuffer.size() != sendBuffer.size()
		|| 0 != memcmp(&receiveBuffer[0], &sendBuffer[0], receiveBuffer.size()))
	{
		THROW_ERROR("Invalid echo response");
	}
}

sockaddr_in QueryBind(SOCKET s)
{
	sockaddr_in bind = { 0 };
	int bindSize = sizeof(bind);

	if (SOCKET_ERROR == getsockname(s, (sockaddr*)&bind, &bindSize))
	{
		const auto err = std::string("Failed to query bind: ")
			.append(FormatWsaError(WSAGetLastError()));

		THROW_ERROR(err.c_str());
	}

	if (bindSize != sizeof(bind))
	{
		THROW_ERROR("Invalid data returned for bind query");
	}

	return bind;
}

void ValidateBind(SOCKET s, const IN_ADDR &ip)
{
	auto actualBind = QueryBind(s);

	if (actualBind.sin_addr.s_addr != ip.s_addr)
	{
		std::wstringstream ss;

		ss << L"Unexpected socket bind. Expected address " << IpToString(ip)
			<< L", Actual address " << IpToString(actualBind.sin_addr);

		THROW_ERROR(common::string::ToAnsi(ss.str()).c_str());
	}
}

void SetSocketRecvTimeout(SOCKET s, std::chrono::milliseconds timeout)
{
	DWORD rawTimeout = static_cast<DWORD>(timeout.count());

	const auto status = setsockopt(s, SOL_SOCKET, SO_RCVTIMEO, reinterpret_cast<char*>(&rawTimeout), sizeof(rawTimeout));

	if (SOCKET_ERROR == status)
	{
		const auto errorCode = WSAGetLastError();

		const auto err = std::string("Failed to set socket recv timeout: ")
			.append(FormatWsaError(errorCode));

		THROW_ERROR(err.c_str());
	}
}

SOCKET CreateBindOverlappedSocket(const IN_ADDR &ip, uint16_t port, bool tcp)
{
	//
	// Turns out all sockets on Windows support overlapped operations.
	//

	return CreateBindSocket(ip, port, tcp);
}

SOCKET CreateBindOverlappedSocket(const std::wstring &ip, uint16_t port, bool tcp)
{
	return CreateBindOverlappedSocket(ParseIpv4(ip), port, tcp);
}

WinsockOverlapped *AllocateWinsockOverlapped()
{
	auto ctx = new WinsockOverlapped;

	ZeroMemory(&ctx->overlapped, sizeof(ctx->overlapped));

	ctx->overlapped.hEvent =  WSACreateEvent();

	ZeroMemory(&ctx->winsockBuffer, sizeof(ctx->winsockBuffer));

	ctx->pendingOperation = false;

	return ctx;
}

void DeleteWinsockOverlapped(WinsockOverlapped **ctx)
{
	if ((*ctx)->pendingOperation)
	{
		WaitForSingleObject((*ctx)->overlapped.hEvent, INFINITE);
	}

	WSACloseEvent((*ctx)->overlapped.hEvent);

	delete *ctx;

	*ctx = nullptr;
}

void AssignOverlappedBuffer(WinsockOverlapped &ctx, std::vector<uint8_t> &&buffer)
{
	assert(!ctx.pendingOperation);

	ctx.buffer.swap(buffer);

	ctx.winsockBuffer.buf = reinterpret_cast<CHAR*>(&ctx.buffer[0]);
	ctx.winsockBuffer.len = static_cast<ULONG>(ctx.buffer.size());
}

void SendOverlappedSocket(SOCKET s, WinsockOverlapped &ctx)
{
	assert(!ctx.pendingOperation);

	WSAResetEvent(ctx.overlapped.hEvent);

	const auto status = WSASend(s, &ctx.winsockBuffer, 1, nullptr, 0, &ctx.overlapped, nullptr);

	if (0 == status || (SOCKET_ERROR == status && WSA_IO_PENDING == WSAGetLastError()))
	{
		ctx.pendingOperation = true;

		return;
	}

	THROW_WINDOWS_ERROR(WSAGetLastError(), "WSASend");
}

void RecvOverlappedSocket(SOCKET s, WinsockOverlapped &ctx, size_t bytes)
{
	assert(!ctx.pendingOperation);

	WSAResetEvent(ctx.overlapped.hEvent);

	if (ctx.winsockBuffer.len != bytes)
	{
		ctx.winsockBuffer.len = static_cast<ULONG>(bytes);

		if (ctx.buffer.size() < bytes)
		{
			ctx.buffer.resize(bytes);
			ctx.winsockBuffer.buf = reinterpret_cast<CHAR*>(&ctx.buffer[0]);
		}
	}

	DWORD flags = 0;

	const auto status = WSARecv(s, &ctx.winsockBuffer, 1, nullptr, &flags, &ctx.overlapped, nullptr);

	if (0 == status || (SOCKET_ERROR == status && WSA_IO_PENDING == WSAGetLastError()))
	{
		ctx.pendingOperation = true;

		return;
	}

	THROW_WINDOWS_ERROR(WSAGetLastError(), "WSARecv");
}

bool PollOverlappedSend(SOCKET s, WinsockOverlapped &ctx)
{
	assert(ctx.pendingOperation);

	DWORD bytesTransferred;
	DWORD flags;

	const auto status = WSAGetOverlappedResult(s, &ctx.overlapped, &bytesTransferred, FALSE, &flags);

	if (FALSE == status)
	{
		if (WSA_IO_INCOMPLETE == WSAGetLastError())
		{
			return false;
		}

		THROW_WINDOWS_ERROR(WSAGetLastError(), "Overlapped send");
	}

	ctx.pendingOperation = false;

	if (bytesTransferred != ctx.winsockBuffer.len)
	{
		THROW_ERROR("Overlapped send completed but did not transfer all bytes");
	}

	return true;
}

bool PollOverlappedRecv(SOCKET s, WinsockOverlapped &ctx)
{
	assert(ctx.pendingOperation);

	DWORD bytesTransferred;
	DWORD flags;

	const auto status = WSAGetOverlappedResult(s, &ctx.overlapped, &bytesTransferred, FALSE, &flags);

	if (FALSE == status)
	{
		if (WSA_IO_INCOMPLETE == WSAGetLastError())
		{
			return false;
		}

		THROW_WINDOWS_ERROR(WSAGetLastError(), "Overlapped receive");
	}

	ctx.pendingOperation = false;

	ctx.winsockBuffer.len = bytesTransferred;

	return true;
}
