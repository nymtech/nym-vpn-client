// Copyright 2016-2026 Mullvad VPN AB. All Rights Reserved.
// Copyright 2026 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

#pragma once

// Include this to get IN6_ADDR
// There's some magical include order which is non-trivial to reproduce
#include <libcommon/network/adapters.h>

#include "settings.h"
#include <string>
#include <cstdint>
#include <optional>
#include <filesystem>

class RuntimeSettings
{

	RuntimeSettings(Settings settings);

public:

	static std::filesystem::path GetSettingsFilePath();

	static void OverrideSettingsFilePath(const std::filesystem::path &path);

	static RuntimeSettings &Instance();

	IN_ADDR tunnelIp();
	IN6_ADDR tunnelIp6();
	IN_ADDR lanIp();
	IN6_ADDR lanIp6();

	//
	// TODO: Start making use of this in combination with the tcpbin info service.
	//
	IN_ADDR publicNonVpnIp();

	IN_ADDR tcpbinServerIp();
	uint16_t tcpbinEchoPort();
	uint16_t tcpbinEchoPortUdp();
	uint16_t tcpbinInfoPort();

private:

	Settings m_settings;

	std::optional<IN_ADDR> m_tunnelIp;
	std::optional<IN6_ADDR> m_tunnelIp6;
	std::optional<IN_ADDR> m_lanIp;
	std::optional<IN6_ADDR> m_lanIp6;

	std::optional<IN_ADDR> m_publicNonVpnIp;

	std::optional<IN_ADDR> m_tcpbinServerIp;
	std::optional<uint16_t> m_tcpbinEchoPort;
	std::optional<uint16_t> m_tcpbinEchoPortUdp;
	std::optional<uint16_t> m_tcpbinInfoPort;
};
