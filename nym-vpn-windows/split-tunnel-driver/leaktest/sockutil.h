// Copyright 2016-2026 Mullvad VPN AB. All Rights Reserved.
// Copyright 2026 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

#pragma once

#include <cstdint>
#include <string>
#include <vector>
#include <chrono>
#include <ws2tcpip.h>

std::string FormatWsaError(int errorCode);

SOCKET CreateBindSocket(const IN_ADDR &ip, uint16_t port = 0, bool tcp = true);

SOCKET CreateBindSocket(const std::wstring &ip, uint16_t port = 0, bool tcp = true);

SOCKET CreateSocket(bool tcp = true);

void ShutdownSocket(SOCKET &s);

void ConnectSocket(SOCKET s, const IN_ADDR &ip, uint16_t port);

void ConnectSocket(SOCKET s, const std::wstring &ip, uint16_t port);

std::vector<uint8_t> SendRecvSocket(SOCKET s, const std::vector<uint8_t> &sendBuffer);

void SendRecvValidateEcho(SOCKET s, const std::vector<uint8_t> &sendBuffer);

sockaddr_in QueryBind(SOCKET s);

void ValidateBind(SOCKET s, const IN_ADDR &ip);

void SetSocketRecvTimeout(SOCKET s, std::chrono::milliseconds timeout);

SOCKET CreateBindOverlappedSocket(const IN_ADDR &ip, uint16_t port = 0, bool tcp = true);

SOCKET CreateBindOverlappedSocket(const std::wstring &ip, uint16_t port = 0, bool tcp = true);

struct WinsockOverlapped
{
	//
	// Overlapped instance with valid event.
	//
	WSAOVERLAPPED overlapped;

	//
	// Actual data buffer.
	//
	std::vector<uint8_t> buffer;

	//
	// Buffer descriptor.
	//
	WSABUF winsockBuffer;

	//
	// Whether there is an active send or receive.
	//
	bool pendingOperation;
};

WinsockOverlapped *AllocateWinsockOverlapped();

void DeleteWinsockOverlapped(WinsockOverlapped **ctx);

void AssignOverlappedBuffer(WinsockOverlapped &ctx, std::vector<uint8_t> &&buffer);

void SendOverlappedSocket(SOCKET s, WinsockOverlapped &ctx);

void RecvOverlappedSocket(SOCKET s, WinsockOverlapped &ctx, size_t bytes = 0);

bool PollOverlappedSend(SOCKET s, WinsockOverlapped &ctx);

bool PollOverlappedRecv(SOCKET s, WinsockOverlapped &ctx);
