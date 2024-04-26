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
import net.nymtech.vpn.util.InvalidCredentialException
import timber.log.Timber
import javax.inject.Inject
import javax.inject.Provider

@AndroidEntryPoint
class BootReceiver : BroadcastReceiver() {

	@Inject
	lateinit var settingsRepository: SettingsRepository

	@Inject
	lateinit var secretsRepository: Provider<SecretsRepository>

	@Inject
	lateinit var vpnClient: Provider<VpnClient>

	override fun onReceive(context: Context?, intent: Intent?) = goAsync {
		if (Intent.ACTION_BOOT_COMPLETED != intent?.action) return@goAsync
		if (settingsRepository.isAutoStartEnabled()) {
			val entryCountry = settingsRepository.getFirstHopCountry()
			val exitCountry = settingsRepository.getLastHopCountry()
			val credential = secretsRepository.get().getCredential()
			val mode = settingsRepository.getVpnMode()
			if (credential != null) {
				context?.let { context ->
					val entry = entryCountry.toEntryPoint()
					val exit = exitCountry.toExitPoint()
					try {
						vpnClient.get().apply {
							this.mode = mode
							this.exitPoint = exit
							this.entryPoint = entry
						}.start(context, credential, true)
						NymVpn.requestTileServiceStateUpdate()
					} catch (e: InvalidCredentialException) {
						Timber.w(e)
					}
				}
			}
		}
	}
}
