package net.nymtech.nymvpn.receiver

import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent
import dagger.hilt.android.AndroidEntryPoint
import net.nymtech.nymvpn.NymVpn
import net.nymtech.nymvpn.data.GatewayRepository
import net.nymtech.nymvpn.data.SettingsRepository
import net.nymtech.nymvpn.util.goAsync
import net.nymtech.vpn.NymVpnClient
import javax.inject.Inject

@AndroidEntryPoint
class BootReceiver : BroadcastReceiver() {
	@Inject
	lateinit var gatewayRepository: GatewayRepository

	@Inject
	lateinit var settingsRepository: SettingsRepository

	override fun onReceive(context: Context?, intent: Intent?) = goAsync {
		if (Intent.ACTION_BOOT_COMPLETED != intent?.action) return@goAsync
		if (settingsRepository.isAutoStartEnabled()) {
			val entryCountry = gatewayRepository.getFirstHopCountry()
			val exitCountry = gatewayRepository.getLastHopCountry()
			val mode = settingsRepository.getVpnMode()
			context?.let { context ->
				val entry = entryCountry.toEntryPoint()
				val exit = exitCountry.toExitPoint()
				NymVpnClient.configure(entry, exit, mode)
				NymVpnClient.start(context)
				NymVpn.requestTileServiceStateUpdate(context)
			}
		}
	}
}
