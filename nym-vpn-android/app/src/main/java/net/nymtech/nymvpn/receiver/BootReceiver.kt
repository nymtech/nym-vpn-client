package net.nymtech.nymvpn.receiver

import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent
import dagger.hilt.android.AndroidEntryPoint
import net.nymtech.nymvpn.NymVpn
import net.nymtech.nymvpn.data.SecretsRepository
import net.nymtech.nymvpn.data.SettingsRepository
import net.nymtech.nymvpn.util.goAsync
import net.nymtech.vpn.VpnClient
import javax.inject.Inject

@AndroidEntryPoint
class BootReceiver : BroadcastReceiver() {

	@Inject
	lateinit var settingsRepository: SettingsRepository

	@Inject
	lateinit var secretsRepository: SecretsRepository

	@Inject
	lateinit var vpnClient: VpnClient

	override fun onReceive(context: Context?, intent: Intent?) = goAsync {
		if (Intent.ACTION_BOOT_COMPLETED != intent?.action) return@goAsync
		if (settingsRepository.isAutoStartEnabled()) {
			val entryCountry = settingsRepository.getFirstHopCountry()
			val exitCountry = settingsRepository.getLastHopCountry()
			val credential = secretsRepository.getCredential()
			val mode = settingsRepository.getVpnMode()
			if (credential != null) {
				context?.let { context ->
					val entry = entryCountry.toEntryPoint()
					val exit = exitCountry.toExitPoint()
					vpnClient.apply {
						this.mode = mode
						this.exitPoint = exit
						this.entryPoint = entry
					}.start(context, credential, true)
					NymVpn.requestTileServiceStateUpdate(context)
				}
			}
		}
	}
}
