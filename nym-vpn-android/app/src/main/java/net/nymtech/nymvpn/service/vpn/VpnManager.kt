package net.nymtech.nymvpn.service.vpn

import android.content.Context

interface VpnManager {

	fun stopVpn(context: Context, foreground: Boolean)
	suspend fun startVpn(context: Context, foreground: Boolean): Result<Unit>
}
