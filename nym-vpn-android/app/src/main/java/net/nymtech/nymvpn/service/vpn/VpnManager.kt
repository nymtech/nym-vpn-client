package net.nymtech.nymvpn.service.vpn

interface VpnManager {

	suspend fun stopVpn(foreground: Boolean)
	suspend fun startVpn(foreground: Boolean): Result<Unit>
}
