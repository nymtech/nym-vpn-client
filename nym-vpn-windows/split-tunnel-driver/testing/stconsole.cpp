// Copyright 2016-2026 Mullvad VPN AB. All Rights Reserved.
// Copyright 2026 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

#include <libcommon/network/adapters.h>
#include <libcommon/string.h>
#include <libcommon/error.h>

#include <iostream>
#include <string>
#include <vector>
#include <sstream>
#include <iomanip>
#include <windows.h>
#include <conio.h>
#include <ip2string.h>
#include <winternl.h>
#include <ws2ipdef.h>
#include <process.h>
#include "proc.h"

#include "../nymvpn-split-tunnel/public.h"

#pragma comment(lib, "iphlpapi.lib")


static const wchar_t DriverSymbolicName[] = L"\\\\.\\NYMVPNSPLITTUNNEL";
HANDLE g_DriverHandle = INVALID_HANDLE_VALUE;

std::vector<std::wstring> g_imagenames;

bool g_DisplayEvents = false;

bool SendIoControl(DWORD code, void *inBuffer, DWORD inBufferSize,
	void *outBuffer, DWORD outBufferSize, DWORD *bytesReturned)
{
	OVERLAPPED o = { 0 };

	//
	// Event should not be created on-the-fly.
	//
	// Create an event for each thread that needs to send a request
	// and keep the event around.
	//
	o.hEvent = CreateEventW(nullptr, TRUE, FALSE, nullptr);

	auto status = DeviceIoControl(g_DriverHandle, code,
		inBuffer, inBufferSize, outBuffer, outBufferSize, bytesReturned, &o);

	if (FALSE != status)
	{
		CloseHandle(o.hEvent);

		return true;
	}

	if (ERROR_IO_PENDING != GetLastError())
	{
		//THROW_ERROR("Could not post request to driver");

		CloseHandle(o.hEvent);

		return false;
	}

	DWORD tempBytesReturned = 0;

	status = GetOverlappedResult(g_DriverHandle, &o, &tempBytesReturned, TRUE);

	CloseHandle(o.hEvent);

	if (FALSE == status)
	{
		//THROW_ERROR("Failed to wait on driver to complete request");

		return false;
	}

	*bytesReturned = tempBytesReturned;

	return true;
}

ST_DRIVER_STATE GetDriverState()
{
	if (INVALID_HANDLE_VALUE == g_DriverHandle)
	{
		THROW_ERROR("Not connected to driver");
	}

	DWORD bytesReturned;

	SIZE_T buffer;

	auto status = SendIoControl((DWORD)IOCTL_ST_GET_STATE, nullptr, 0, &buffer, sizeof(buffer), &bytesReturned);

	if (!status)
	{
		THROW_ERROR("Failed to request state info from driver");
	}

	return static_cast<ST_DRIVER_STATE>(buffer);
}

std::wstring MapDriverState(ST_DRIVER_STATE state)
{
	switch (state)
	{
		case ST_DRIVER_STATE_STARTED: return L"ST_DRIVER_STATE_STARTED";
		case ST_DRIVER_STATE_INITIALIZED: return L"ST_DRIVER_STATE_INITIALIZED";
		case ST_DRIVER_STATE_READY: return L"ST_DRIVER_STATE_READY";
		case ST_DRIVER_STATE_ENGAGED: return L"ST_DRIVER_STATE_ENGAGED";
		case ST_DRIVER_STATE_ZOMBIE: return L"ST_DRIVER_STATE_ZOMBIE";
		default:
		{
			THROW_ERROR("Unknown driver state");
		}
	}
}

void ProcessConnect()
{
	g_DriverHandle = CreateFileW(DriverSymbolicName, GENERIC_READ | GENERIC_WRITE,
		0, nullptr, OPEN_EXISTING, FILE_FLAG_OVERLAPPED, nullptr);

	if (INVALID_HANDLE_VALUE == g_DriverHandle)
	{
		THROW_WINDOWS_ERROR(GetLastError(), "Connect to driver");
	}

	std::wcout << L"Driver state: " << MapDriverState(GetDriverState()) << std::endl;

	std::wcout << L"Successfully connected to driver" << std::endl;
}

void ProcessInitialize()
{
	DWORD bytesReturned;

	auto status = SendIoControl((DWORD)IOCTL_ST_INITIALIZE, nullptr, 0, nullptr, 0, &bytesReturned);

	if (!status)
	{
		THROW_ERROR("Initialization command failed");
	}

	std::wcout << L"Driver state: " << MapDriverState(GetDriverState()) << std::endl;

	std::wcout << L"Successfully initialized driver" << std::endl;
}

std::vector<uint8_t> MakeConfiguration(const std::vector<std::wstring> &imageNames)
{
	size_t totalStringLength = 0;

	for (const auto &imageName : imageNames)
	{
		totalStringLength += imageName.size() * sizeof(wchar_t);
	}

	size_t totalBufferSize = sizeof(ST_CONFIGURATION_HEADER)
		+ (sizeof(ST_CONFIGURATION_ENTRY) * imageNames.size())
		+ totalStringLength;

	std::vector<uint8_t> buffer(totalBufferSize);

	auto header = (ST_CONFIGURATION_HEADER*)&buffer[0];
	auto entry = (ST_CONFIGURATION_ENTRY*)(header + 1);

	auto stringDest = &buffer[0] + sizeof(ST_CONFIGURATION_HEADER)
		+ (sizeof(ST_CONFIGURATION_ENTRY) * imageNames.size());

	SIZE_T stringOffset = 0;

	for (const auto &imageName : imageNames)
	{
		auto stringLength = imageName.size() * sizeof(wchar_t);

		entry->ImageNameLength = (USHORT)stringLength;
		entry->ImageNameOffset = stringOffset;

		memcpy(stringDest, imageName.c_str(), stringLength);

		++entry;
		stringDest += stringLength;
		stringOffset += stringLength;
	}

	header->NumEntries = imageNames.size();
	header->TotalLength = totalBufferSize;

	return buffer;
}

void ProcessSetConfig(const std::vector<std::wstring> &imageNames)
{
	if (INVALID_HANDLE_VALUE == g_DriverHandle)
	{
		THROW_ERROR("Not connected to driver");
	}

	std::wcout << L"Sending the following config to driver:" << std::endl;

	for (const auto &imagename : imageNames)
	{
		std::wcout << L"  " << imagename << std::endl;
	}

	auto blob = MakeConfiguration(imageNames);

	DWORD bytesReturned;

	auto status = SendIoControl((DWORD)IOCTL_ST_SET_CONFIGURATION,
		&blob[0], (DWORD)blob.size(), nullptr, 0, &bytesReturned);

	if (!status)
	{
		THROW_ERROR("Set configuration");
	}

	std::wcout << L"Driver state: " << MapDriverState(GetDriverState()) << std::endl;

	std::wcout << L"Successfully set configuration" << std::endl;
}

void ProcessAddConfig(const std::wstring &imageName)
{
	auto tempNames = g_imagenames;

	tempNames.push_back(imageName);

	ProcessSetConfig(tempNames);

	// Persist data now that the above call did not throw.
	g_imagenames = tempNames;
}

void ProcessClearConfig();

void ProcessRemoveConfig(const std::wstring &imageName)
{
	auto iterMatch = std::find_if(g_imagenames.begin(), g_imagenames.end(), [&imageName](const std::wstring &candidate)
	{
		return 0 == _wcsicmp(candidate.c_str(), imageName.c_str());
	});

	if (iterMatch == g_imagenames.end())
	{
		THROW_ERROR("Specified imagename was not previously registered");
	}

	auto indexMatch = std::distance(g_imagenames.begin(), iterMatch);

	auto tempNames = g_imagenames;

	tempNames.erase(tempNames.begin() + indexMatch);

	if (tempNames.empty())
	{
		ProcessClearConfig();

		return;
	}

	ProcessSetConfig(tempNames);

	// Persist data now that the above call did not throw.
	g_imagenames = tempNames;
}

void ProcessGetConfig()
{
	if (INVALID_HANDLE_VALUE == g_DriverHandle)
	{
		THROW_ERROR("Not connected to driver");
	}

	DWORD bytesReturned;

	SIZE_T requiredBufferSize;

	auto status = SendIoControl((DWORD)IOCTL_ST_GET_CONFIGURATION,
		nullptr, 0, &requiredBufferSize, sizeof(requiredBufferSize), &bytesReturned);

	if (!status || 0 == bytesReturned)
	{
		THROW_ERROR("Get configuration");
	}

	std::vector<uint8_t> buffer(requiredBufferSize, 0);

	status = SendIoControl((DWORD)IOCTL_ST_GET_CONFIGURATION,
		nullptr, 0, &buffer[0], (DWORD)buffer.size(), &bytesReturned);

	if (!status || bytesReturned != buffer.size())
	{
		THROW_ERROR("Get configuration");
	}

	auto header = (ST_CONFIGURATION_HEADER*)&buffer[0];
	auto entry = (ST_CONFIGURATION_ENTRY*)(header + 1);

	auto stringBuffer = (uint8_t *)(entry + header->NumEntries);

	std::vector<std::wstring> imageNames;

	for (auto i = 0; i < header->NumEntries; ++i, ++entry)
	{
		imageNames.emplace_back
		(
			(wchar_t*)(stringBuffer + entry->ImageNameOffset),
			(wchar_t*)(stringBuffer + entry->ImageNameOffset + entry->ImageNameLength)
		);
	}

	std::wcout << L"Successfully got configuration" << std::endl;

	std::wcout << L"Image names in config:" << std::endl;

	for (const auto &imageName : imageNames)
	{
		std::wcout << L"  " << imageName << std::endl;
	}
}

void ProcessClearConfig()
{
	if (INVALID_HANDLE_VALUE == g_DriverHandle)
	{
		THROW_ERROR("Not connected to driver");
	}

	DWORD bytesReturned;

	auto status = SendIoControl((DWORD)IOCTL_ST_CLEAR_CONFIGURATION,
		nullptr, 0, nullptr, 0, &bytesReturned);

	if (!status)
	{
		THROW_ERROR("Clear configuration");
	}

	g_imagenames.clear();

	std::wcout << L"Driver state: " << MapDriverState(GetDriverState()) << std::endl;

	std::wcout << L"Successfully cleared configuration" << std::endl;
}

void ProcessRegisterProcesses()
{
	if (INVALID_HANDLE_VALUE == g_DriverHandle)
	{
		THROW_ERROR("Not connected to driver");
	}

	auto blob = BuildRegisterProcessesPayload();

	DWORD bytesReturned;

	auto status = SendIoControl((DWORD)IOCTL_ST_REGISTER_PROCESSES,
		&blob[0], (DWORD)blob.size(), nullptr, 0, &bytesReturned);

	if (!status)
	{
		THROW_ERROR("Register processes");
	}

	std::wcout << L"Driver state: " << MapDriverState(GetDriverState()) << std::endl;

	std::wcout << L"Successfully registered processes" << std::endl;
}

void GetAdapterAddresses(const std::wstring &adapterName, IN_ADDR &ipv4, IN6_ADDR &ipv6)
{
	const DWORD flags = GAA_FLAG_SKIP_ANYCAST | GAA_FLAG_SKIP_MULTICAST | GAA_FLAG_SKIP_DNS_SERVER;

	common::network::Adapters adapters(AF_INET, flags);

	bool ipv4Done = false;

	for (auto adapter = adapters.next(); adapter != NULL; adapter = adapters.next())
	{
		if (0 != _wcsicmp(adapter->FriendlyName, adapterName.c_str()))
		{
			continue;
		}

		if (adapter->Ipv4Enabled == 0
			|| adapter->FirstUnicastAddress == nullptr)
		{
			break;
		}

		auto sa = (SOCKADDR_IN*)adapter->FirstUnicastAddress->Address.lpSockaddr;
		ipv4 = sa->sin_addr;

		ipv4Done = true;
		break;
	}

	if (!ipv4Done)
	{
		throw std::runtime_error("Could not determine adapter IPv4 address");
	}

	common::network::Adapters adapters6(AF_INET6, flags);

	bool ipv6Done = false;

	for (auto adapter = adapters6.next(); adapter != NULL; adapter = adapters6.next())
	{
		if (0 != _wcsicmp(adapter->FriendlyName, adapterName.c_str()))
		{
			continue;
		}

		if (adapter->Ipv6Enabled == 0
			|| adapter->FirstUnicastAddress == nullptr)
		{
			break;
		}

		auto sa = (SOCKADDR_IN6*)adapter->FirstUnicastAddress->Address.lpSockaddr;
		ipv6 = sa->sin6_addr;

		ipv6Done = true;
		break;
	}

	if (!ipv6Done)
	{
		throw std::runtime_error("Could not determine adapter IPv6 address");
	}
}

std::vector<uint8_t> BuildRegisterIpsPayload()
{
	std::vector<uint8_t> payload(sizeof(ST_IP_ADDRESSES));

	auto ip = reinterpret_cast<ST_IP_ADDRESSES*>(&payload[0]);

	GetAdapterAddresses(L"Ethernet", ip->InternetIpv4, ip->InternetIpv6);
	GetAdapterAddresses(L"Mullvad", ip->TunnelIpv4, ip->TunnelIpv6);

	wchar_t stringBuffer[100];

	std::wcout << L"Internet addresses" << std::endl;

	RtlIpv4AddressToStringW(&(ip->InternetIpv4), stringBuffer);
	std::wcout << L"  Ipv4: " << stringBuffer << std::endl;

	RtlIpv6AddressToStringW(&(ip->InternetIpv6), stringBuffer);
	std::wcout << L"  Ipv6: " << stringBuffer << std::endl;

	std::wcout << L"Tunnel addresses" << std::endl;

	RtlIpv4AddressToStringW(&(ip->TunnelIpv4), stringBuffer);
	std::wcout << L"  Ipv4: " << stringBuffer << std::endl;

	RtlIpv6AddressToStringW(&(ip->TunnelIpv6), stringBuffer);
	std::wcout << L"  Ipv6: " << stringBuffer << std::endl;

	//ip->InternetIpv4.S_un.S_addr = 0x0f02000a;
	//ip->InternetIpv6.u.Byte[0] = 0;
	//ip->TunnelIpv4.S_un.S_addr = 0x0c00080a;
	//ip->TunnelIpv6.u.Byte[0] = 0;

	return payload;
}

void ProcessRegisterIps()
{
	if (INVALID_HANDLE_VALUE == g_DriverHandle)
	{
		THROW_ERROR("Not connected to driver");
	}

	auto blob = BuildRegisterIpsPayload();

	DWORD bytesReturned;

	auto status = SendIoControl((DWORD)IOCTL_ST_REGISTER_IP_ADDRESSES,
		&blob[0], (DWORD)blob.size(), nullptr, 0, &bytesReturned);

	if (!status)
	{
		THROW_ERROR("Register IP addresses");
	}

	std::wcout << L"Driver state: " << MapDriverState(GetDriverState()) << std::endl;

	std::wcout << L"Successfully registered IP addresses" << std::endl;
}

void ProcessGetIps()
{
	ST_IP_ADDRESSES ips = { 0 };

	DWORD bytesReturned;

	auto status = SendIoControl((DWORD)IOCTL_ST_GET_IP_ADDRESSES,
		nullptr, 0, &ips, (DWORD)sizeof(ips), &bytesReturned);

	if (!status || bytesReturned != sizeof(ips))
	{
		THROW_ERROR("Register IP addresses");
	}

	std::wcout << L"Internet IPv4: " << common::string::FormatIpv4(ips.InternetIpv4.S_un.S_addr) << std::endl;
	std::wcout << L"Internet IPv6: " << common::string::FormatIpv6(ips.InternetIpv6.u.Byte) << std::endl;
	std::wcout << L"Tunnel IPv4: " << common::string::FormatIpv4(ips.TunnelIpv4.S_un.S_addr) << std::endl;
	std::wcout << L"Tunnel IPv6: " << common::string::FormatIpv6(ips.TunnelIpv6.u.Byte) << std::endl;
}

//
// This is duplicated from proc.cpp
//
HANDLE XxxMakeHandle(DWORD h)
{
	return reinterpret_cast<HANDLE>(static_cast<size_t>(h));
}

DWORD MakeDword(HANDLE h)
{
	return static_cast<DWORD>(reinterpret_cast<size_t>(h));
}

void ProcessQueryProcess(const std::wstring &processId)
{
	ST_QUERY_PROCESS q = { 0 };

	q.ProcessId = XxxMakeHandle(_wtoi(processId.c_str()));

	std::vector<uint8_t> buffer(1024);

	DWORD bytesReturned;

	auto status = SendIoControl((DWORD)IOCTL_ST_QUERY_PROCESS,
		&q, sizeof(q), &buffer[0], (DWORD)buffer.size(), &bytesReturned);

	if (!status)
	{
		THROW_ERROR("Query process");
	}

	//
	// Dump retrieved information.
	//

	buffer.push_back(0);
	buffer.push_back(0);

	auto r = (ST_QUERY_PROCESS_RESPONSE *)&buffer[0];

	std::wcout << L"Process id: " << MakeDword(r->ProcessId) << std::endl;
	std::wcout << L"Parent process id: " << MakeDword(r->ParentProcessId) << std::endl;
	std::wcout << L"Split: " << r->Split << std::endl;
	std::wcout << L"Imagename: " << r->ImageName << std::endl;
}

void ProcessDisplayEvents()
{
	g_DisplayEvents = !g_DisplayEvents;

	std::wcout << L"Displaying events: " << std::boolalpha << g_DisplayEvents << std::endl;
}

void DisplaySplittingEvent(const ST_SPLITTING_EVENT *evt, size_t /*eventSize*/)
{
	std::wcout << L"Process id: " << MakeDword(evt->ProcessId) << std::endl;

	std::wcout << L"Flags:" << std::endl;

	if ((evt->Reason & ST_SPLITTING_REASON_BY_INHERITANCE) != 0)
	{
		std::wcout << L"  ST_SPLITTING_REASON_BY_INHERITANCE" << std::endl;
	}

	if ((evt->Reason & ST_SPLITTING_REASON_BY_CONFIG) != 0)
	{
		std::wcout << L"  ST_SPLITTING_REASON_BY_CONFIG" << std::endl;
	}

	if ((evt->Reason & ST_SPLITTING_REASON_PROCESS_ARRIVING) != 0)
	{
		std::wcout << L"  ST_SPLITTING_REASON_PROCESS_ARRIVING" << std::endl;
	}

	if ((evt->Reason & ST_SPLITTING_REASON_PROCESS_DEPARTING) != 0)
	{
		std::wcout << L"  ST_SPLITTING_REASON_PROCESS_DEPARTING" << std::endl;
	}

	std::wstring imageName(&evt->ImageName[0], &evt->ImageName[0] + (evt->ImageNameLength / sizeof(wchar_t)));

	std::wcout << L"Imagename: " << imageName << std::endl;
}

void DisplaySplittingErrorEvent(const ST_SPLITTING_ERROR_EVENT *evt, size_t /*eventSize*/)
{
	std::wcout << L"Process id: " << MakeDword(evt->ProcessId) << std::endl;

	std::wstring imageName(&evt->ImageName[0], &evt->ImageName[0] + (evt->ImageNameLength / sizeof(wchar_t)));

	std::wcout << L"Imagename: " << imageName << std::endl;
}

void DisplayErrorMessageEvent(const ST_ERROR_MESSAGE_EVENT *evt, size_t /*eventSize*/)
{
	std::wstringstream ss;

	ss << L"Status: 0x" << std::setw(8) << std::setfill(L'0') << std::hex << evt->Status;

	std::wcout << ss.str() << std::endl;

	std::wstring message(&evt->ErrorMessage[0], &evt->ErrorMessage[0] + (evt->ErrorMessageLength / sizeof(wchar_t)));

	std::wcout << L"Error message: " << message << std::endl;
}

void ParseDisplayEvent(const uint8_t *evt, size_t eventSize)
{
	if (!g_DisplayEvents)
	{
		return;
	}

	std::wcout << L"Event received, " << eventSize << " bytes" << std::endl;

	auto header = (ST_EVENT_HEADER *)evt;

	std::wcout << L"Payload size, " << header->EventSize << " bytes" << std::endl;

	switch (header->EventId)
	{
		case ST_EVENT_ID_START_SPLITTING_PROCESS:
		{
			std::wcout << L"Type: ST_EVENT_START_SPLITTING_PROCESS" << std::endl;

			DisplaySplittingEvent((ST_SPLITTING_EVENT*)&header->EventData[0], header->EventSize);

			break;
		}
		case ST_EVENT_ID_STOP_SPLITTING_PROCESS:
		{
			std::wcout << L"Type: ST_EVENT_STOP_SPLITTING_PROCESS" << std::endl;

			DisplaySplittingEvent((ST_SPLITTING_EVENT*)&header->EventData[0], header->EventSize);

			break;
		}
		case ST_EVENT_ID_ERROR_START_SPLITTING_PROCESS:
		{
			std::wcout << L"Type: ST_EVENT_ERROR_START_SPLITTING_PROCESS" << std::endl;

			DisplaySplittingErrorEvent((ST_SPLITTING_ERROR_EVENT*)&header->EventData[0], header->EventSize);

			break;
		}
		case ST_EVENT_ID_ERROR_STOP_SPLITTING_PROCESS:
		{
			std::wcout << L"Type: ST_EVENT_ERROR_STOP_SPLITTING_PROCESS" << std::endl;

			DisplaySplittingErrorEvent((ST_SPLITTING_ERROR_EVENT*)&header->EventData[0], header->EventSize);

			break;
		}
		case ST_EVENT_ID_ERROR_MESSAGE:
		{
			std::wcout << L"Type: ST_EVENT_ID_ERROR_MESSAGE" << std::endl;

			DisplayErrorMessageEvent((ST_ERROR_MESSAGE_EVENT*)&header->EventData[0], header->EventSize);

			break;
		}
		default:
		{
			std::wcout << L"Unsupported event" << std::endl;
		}
	}
}

unsigned __stdcall EventThread(void * /*rawContext*/)
{
	//
	// Wait for connect
	//

	for (;;)
	{
		if (g_DriverHandle == INVALID_HANDLE_VALUE)
		{
			Sleep(1000);

			continue;
		}

		break;
	}

	//
	// Continously issue event requests
	//

	std::vector<uint8_t> buffer(2048);

	for (;;)
	{
		DWORD bytesReturned;

		auto status = SendIoControl((DWORD)IOCTL_ST_DEQUEUE_EVENT, nullptr, 0,
			&buffer[0], static_cast<DWORD>(buffer.size()), &bytesReturned);

		if (!status)
		{
			//std::wcout << L"Failed to dequeue event from driver" << std::endl;

			Sleep(1000);

			continue;
		}

		ParseDisplayEvent(&buffer[0], bytesReturned);
	}
}

void ResetDriver()
{
	DWORD bytesReturned;

	auto status = SendIoControl((DWORD)IOCTL_ST_RESET,
		nullptr, 0, nullptr, 0, &bytesReturned);

	if (!status)
	{
		THROW_ERROR("Request to reset driver has failed");
	}

	std::wcout << L"Driver state: " << MapDriverState(GetDriverState()) << std::endl;
}

bool CreateEventThread()
{
	auto t = _beginthreadex(nullptr, 0, EventThread, nullptr, 0, nullptr);

	auto threadHandle = reinterpret_cast<HANDLE>(t);

	if (0 == threadHandle)
	{
		return false;
	}

	CloseHandle(threadHandle);

	return true;
}

int main()
{
	std::wcout << L"Testing console for split tunnel driver" << std::endl;

	if (!CreateEventThread())
	{
		std::wcout << L"Failed to create event thread" << std::endl;
		return 1;
	}

	for (;;)
	{
		std::wcout << L"cmd> ";

		std::wstring request;
		std::getline(std::wcin, request);

		auto tokens = common::string::Tokenize(request, L" ");

		if (tokens.empty())
		{
			continue;
		}

		if (0 == _wcsicmp(tokens[0].c_str(), L"quit"))
		{
			break;
		}

		try
		{
			if (0 == _wcsicmp(tokens[0].c_str(), L"connect"))
			{
				ProcessConnect();
				continue;
			}

			if (0 == _wcsicmp(tokens[0].c_str(), L"initialize"))
			{
				ProcessInitialize();
				continue;
			}

			if (0 == _wcsicmp(tokens[0].c_str(), L"get-state"))
			{
				std::wcout << L"Driver state: " << MapDriverState(GetDriverState()) << std::endl;
				continue;
			}

			if (0 == _wcsicmp(tokens[0].c_str(), L"add-config"))
			{
				//
				// tokens[1] will be a partial path if the path contains spaces.
				// reuse the source for "tokens" instead.
				//

				ProcessAddConfig(request.substr(sizeof("add-config")));
				continue;
			}

			if (0 == _wcsicmp(tokens[0].c_str(), L"remove-config"))
			{
				ProcessRemoveConfig(request.substr(sizeof("remove-config")));
				continue;
			}

			if (0 == _wcsicmp(tokens[0].c_str(), L"get-config"))
			{
				ProcessGetConfig();
				continue;
			}

			if (0 == _wcsicmp(tokens[0].c_str(), L"clear-config"))
			{
				ProcessClearConfig();
				continue;
			}

			if (0 == _wcsicmp(tokens[0].c_str(), L"register-processes"))
			{
				ProcessRegisterProcesses();
				continue;
			}

			if (0 == _wcsicmp(tokens[0].c_str(), L"register-ips"))
			{
				ProcessRegisterIps();
				continue;
			}

			if (0 == _wcsicmp(tokens[0].c_str(), L"get-ips"))
			{
				ProcessGetIps();
				continue;
			}

			if (0 == _wcsicmp(tokens[0].c_str(), L"dry-run-ips"))
			{
				BuildRegisterIpsPayload();
				continue;
			}

			if (0 == _wcsicmp(tokens[0].c_str(), L"query-process"))
			{
				ProcessQueryProcess(tokens[1]);
				continue;
			}

			if (0 == _wcsicmp(tokens[0].c_str(), L"quick"))
			{
				if (g_DriverHandle != INVALID_HANDLE_VALUE)
				{
					std::wcout << L"Already initialized" << std::endl;

					continue;
				}

				ProcessConnect();
				ProcessInitialize();
				ProcessRegisterProcesses();
				ProcessRegisterIps();

				continue;
			}

			if (0 == _wcsicmp(tokens[0].c_str(), L"display-events"))
			{
				ProcessDisplayEvents();
				continue;
			}

			if (0 == _wcsicmp(tokens[0].c_str(), L"split-firefox"))
			{
				static bool split = false;

				const auto targetSplit = !split;

				std::wcout << L"Splitting firefox: " << std::boolalpha << targetSplit << std::endl;

				const std::wstring path = L"\\Device\\HarddiskVolume2\\Program Files (x86)\\Mozilla Firefox\\firefox.exe";

				if (targetSplit)
				{
					ProcessAddConfig(path);
				}
				else
				{
					ProcessRemoveConfig(path);
				}

				// Safe to update now since above calls did not throw.
				split = targetSplit;

				continue;
			}

			if (0 == _wcsicmp(tokens[0].c_str(), L"reset"))
			{
				ResetDriver();
				continue;
			}
		}
		catch (const std::exception &ex)
		{
			std::cout << "Error: " << ex.what() << std::endl;
			continue;
		}

		std::wcout << L"invalid command" << std::endl;
	}

	if (g_DriverHandle != INVALID_HANDLE_VALUE)
	{
		CloseHandle(g_DriverHandle);
	}

	return 0;
}
