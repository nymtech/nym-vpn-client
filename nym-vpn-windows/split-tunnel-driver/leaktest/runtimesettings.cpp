// Copyright 2016-2026 Mullvad VPN AB. All Rights Reserved.
// Copyright 2026 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

#include "runtimesettings.h"
#include "util.h"
#include <libcommon/error.h>
#include <ws2tcpip.h>
#include <vector>

namespace
{

std::filesystem::path g_SettingsFilePathOverride;

} // anonymous namespace

RuntimeSettings::RuntimeSettings(Settings settings)
	: m_settings(std::move(settings))
{
}

//static
std::filesystem::path RuntimeSettings::GetSettingsFilePath()
{
	if (!g_SettingsFilePathOverride.empty())
	{
		return g_SettingsFilePathOverride;
	}

	wchar_t *rawPath;

	if (0 != _get_wpgmptr(&rawPath))
	{
		throw std::runtime_error(__FUNCTION__ ": _get_wpgmptr choked");
	}

	std::filesystem::path path(rawPath);

	if (!path.is_absolute())
	{
		throw std::runtime_error("Could not construct path for settings file");
	}

	path.replace_filename(L"leaktest.settings");

	return path;
}

//static
void RuntimeSettings::OverrideSettingsFilePath(const std::filesystem::path &path)
{
	g_SettingsFilePathOverride = path;
}

//static
RuntimeSettings &RuntimeSettings::Instance()
{
	//
	// This is fine for testing code.
	//
	// Lazy construction and lazy evaluation means a test that doesn't rely on settings
	// can be ran without creating a settings file.
	//

	static RuntimeSettings *Instance = nullptr;

	if (Instance == nullptr)
	{
		Instance = new RuntimeSettings(Settings::FromFile(GetSettingsFilePath()));
	}

	return *Instance;
}

IN_ADDR RuntimeSettings::tunnelIp()
{
	if (m_tunnelIp.has_value())
	{
		return m_tunnelIp.value();
	}

	auto adapterName = m_settings.get(L"TunnelAdapter");

	IN_ADDR ipv4;

	GetAdapterAddresses(adapterName, &ipv4, nullptr);

	m_tunnelIp = ipv4;

	return ipv4;
}

IN6_ADDR RuntimeSettings::tunnelIp6()
{
	if (m_tunnelIp6.has_value())
	{
		return m_tunnelIp6.value();
	}

	auto adapterName = m_settings.get(L"TunnelAdapter");

	IN6_ADDR ipv6;

	GetAdapterAddresses(adapterName, nullptr, &ipv6);

	m_tunnelIp6 = ipv6;

	return ipv6;
}

IN_ADDR RuntimeSettings::lanIp()
{
	if (m_lanIp.has_value())
	{
		return m_lanIp.value();
	}

	auto adapterName = m_settings.get(L"LanAdapter");

	IN_ADDR ipv4;

	GetAdapterAddresses(adapterName, &ipv4, nullptr);

	m_lanIp = ipv4;

	return ipv4;
}

IN6_ADDR RuntimeSettings::lanIp6()
{
	if (m_lanIp6.has_value())
	{
		return m_lanIp6.value();
	}

	auto adapterName = m_settings.get(L"LanAdapter");

	IN6_ADDR ipv6;

	GetAdapterAddresses(adapterName, nullptr, &ipv6);

	m_lanIp6 = ipv6;

	return ipv6;
}

IN_ADDR RuntimeSettings::publicNonVpnIp()
{
	if (m_publicNonVpnIp.has_value())
	{
		return m_publicNonVpnIp.value();
	}

	auto ip = m_settings.get(L"PublicNonVpnIp");

	sockaddr_in endpoint = { 0 };

	auto status = InetPtonW(AF_INET, ip.c_str(), &endpoint.sin_addr.s_addr);

	if (status != 1)
	{
		THROW_ERROR("Unable to parse IP address");
	}

	m_publicNonVpnIp = endpoint.sin_addr;

	return endpoint.sin_addr;
}

IN_ADDR RuntimeSettings::tcpbinServerIp()
{
	if (m_tcpbinServerIp.has_value())
	{
		return m_tcpbinServerIp.value();
	}

	auto ip = m_settings.get(L"TcpBinServerIp");

	sockaddr_in endpoint = { 0 };

	auto status = InetPtonW(AF_INET, ip.c_str(), &endpoint.sin_addr.s_addr);

	if (status != 1)
	{
		THROW_ERROR("Unable to parse IP address");
	}

	m_tcpbinServerIp = endpoint.sin_addr;

	return endpoint.sin_addr;
}

uint16_t RuntimeSettings::tcpbinEchoPort()
{
	if (m_tcpbinEchoPort.has_value())
	{
		return m_tcpbinEchoPort.value();
	}

	auto port = m_settings.get(L"TcpBinEchoPort");

	auto numericalPort = common::string::LexicalCast<uint16_t>(port);

	m_tcpbinEchoPort = numericalPort;

	return numericalPort;
}

uint16_t RuntimeSettings::tcpbinEchoPortUdp()
{
	if (m_tcpbinEchoPortUdp.has_value())
	{
		return m_tcpbinEchoPortUdp.value();
	}

	auto port = m_settings.get(L"TcpBinEchoPortUdp");

	auto numericalPort = common::string::LexicalCast<uint16_t>(port);

	m_tcpbinEchoPortUdp = numericalPort;

	return numericalPort;
}

uint16_t RuntimeSettings::tcpbinInfoPort()
{
	if (m_tcpbinInfoPort.has_value())
	{
		return m_tcpbinInfoPort.value();
	}

	auto port = m_settings.get(L"TcpBinInfoPort");

	auto numericalPort = common::string::LexicalCast<uint16_t>(port);

	m_tcpbinInfoPort = numericalPort;

	return numericalPort;
}
