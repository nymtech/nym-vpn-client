// Copyright 2016-2026 Mullvad VPN AB. All Rights Reserved.
// Copyright 2026 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

#pragma once

#include <filesystem>
#include <string>
#include <libcommon/string.h>

using common::string::KeyValuePairs;

class Settings
{
public:

	Settings(KeyValuePairs values)
		: m_values(std::move(values))
	{
	}

	static Settings FromFile(const std::filesystem::path &filename);

	const std::wstring &get(const std::wstring &key);

private:

	KeyValuePairs m_values;
};
