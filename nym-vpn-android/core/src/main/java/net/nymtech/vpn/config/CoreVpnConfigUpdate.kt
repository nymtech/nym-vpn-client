package net.nymtech.vpn.config

import net.nymtech.vpn.backend.Tunnel
import nym_vpn_lib_types.EntryPoint
import nym_vpn_lib_types.ExitPoint

sealed class CoreVpnConfigUpdate {
	data class SetEntryPoint(val value: EntryPoint) : CoreVpnConfigUpdate()
	data class SetExitPoint(val value: ExitPoint) : CoreVpnConfigUpdate()
	data class SetMode(val value: Tunnel.Mode) : CoreVpnConfigUpdate()
	data class SetBypassLan(val value: Boolean) : CoreVpnConfigUpdate()
	data class SetEnableBridges(val value: Boolean) : CoreVpnConfigUpdate()
	data class SetCustomDnsEnabled(val value: Boolean) : CoreVpnConfigUpdate()
	data class SetCustomDns(val value: List<String>) : CoreVpnConfigUpdate()
	data class SetRestrictedApps(val value: List<String>) : CoreVpnConfigUpdate()

	data class SetNetwork(val value: Tunnel.Environment) : CoreVpnConfigUpdate()
	data class SetDebugLog(val value: Boolean) : CoreVpnConfigUpdate()
	data class SetSentry(val value: Boolean) : CoreVpnConfigUpdate()
	data class SetAdBlockingEnabled(val value: Boolean) : CoreVpnConfigUpdate()
	data class SetStealthMode(val value: Boolean) : CoreVpnConfigUpdate()
	data class SetNodeFamiliesNotificationsEnabled(val value: Boolean) : CoreVpnConfigUpdate()
	data class SetGeoExclusionEnabled(val value: Boolean) : CoreVpnConfigUpdate()
	data class SetGeoExclusionPort(val value: Int) : CoreVpnConfigUpdate()
	data class SetGeoExclusionCountries(val value: List<String>) : CoreVpnConfigUpdate()
}
