package net.nymtech.nymvpn.service

import android.content.Intent
import android.os.IBinder
import androidx.lifecycle.LifecycleService
import androidx.lifecycle.lifecycleScope
import dagger.hilt.android.AndroidEntryPoint
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import net.nymtech.nymvpn.NymVpn
import net.nymtech.nymvpn.data.SecretsRepository
import net.nymtech.nymvpn.data.SettingsRepository
import net.nymtech.vpn.VpnClient
import net.nymtech.vpn.util.InvalidCredentialException
import timber.log.Timber
import javax.inject.Inject
import javax.inject.Provider

@AndroidEntryPoint
class AlwaysOnVpnService : LifecycleService() {

	@Inject
	lateinit var settingsRepository: SettingsRepository

	@Inject
	lateinit var secretsRepository: Provider<SecretsRepository>

	@Inject
	lateinit var vpnClient: Provider<VpnClient>

	override fun onBind(intent: Intent): IBinder? {
		super.onBind(intent)
		// We don't provide binding, so return null
		return null
	}

	override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
		if (intent == null || intent.component == null || intent.component!!.packageName != packageName) {
			Timber.i("Always-on VPN requested start")
			lifecycleScope.launch(Dispatchers.IO) {
				val credential = secretsRepository.get().getCredential()
				if (credential != null) {
					val entryCountry = settingsRepository.getFirstHopCountry()
					val exitCountry = settingsRepository.getLastHopCountry()
					val mode = settingsRepository.getVpnMode()
					val entry = entryCountry.toEntryPoint()
					val exit = exitCountry.toExitPoint()
					try {
						vpnClient.get().apply {
							this.mode = mode
							this.entryPoint = entry
							this.exitPoint = exit
						}.start(this@AlwaysOnVpnService, credential, true)
					} catch (e: InvalidCredentialException) {
						Timber.w(e)
					}
					NymVpn.requestTileServiceStateUpdate()
				}
			}
		}
		return super.onStartCommand(intent, flags, startId)
	}
}
