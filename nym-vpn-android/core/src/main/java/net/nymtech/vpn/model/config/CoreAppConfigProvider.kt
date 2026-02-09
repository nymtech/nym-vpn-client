package net.nymtech.vpn.model.config

import nym_vpn_lib_types.UserAgent

interface CoreAppConfigProvider {
	fun getUserAgent(): UserAgent
}
