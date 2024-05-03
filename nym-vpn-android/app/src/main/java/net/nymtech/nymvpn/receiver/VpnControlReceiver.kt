package net.nymtech.nymvpn.receiver

import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent
import net.nymtech.nymvpn.data.SettingsRepository
import net.nymtech.nymvpn.service.vpn.VpnManager
import net.nymtech.nymvpn.util.goAsync
import timber.log.Timber
import javax.inject.Inject

class VpnControlReceiver : BroadcastReceiver() {
	@Inject
	lateinit var settingsRepository: SettingsRepository

	@Inject
	lateinit var vpnManager: VpnManager

	override fun onReceive(context: Context?, intent: Intent?) = goAsync {
		context?.let { context ->
			when (intent?.action) {
				VPN_START_ACTION -> {
					vpnManager.startVpn(context, true).onFailure {
						Timber.w(it)
					}
				}
				VPN_STOP_ACTION -> {
					vpnManager.stopVpn(context, true)
				}
			}
		}
	}
	companion object {
		const val VPN_START_ACTION = "net.nymtech.nymvpn.START"
		const val VPN_STOP_ACTION = "net.nymtech.nymvpn.STOP"
	}
}
