package net.nymtech.nymvpn.receiver

import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent
import dagger.hilt.android.AndroidEntryPoint
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.launch
import net.nymtech.nymvpn.data.SettingsRepository
import net.nymtech.nymvpn.di.qualifiers.ApplicationScope
import net.nymtech.nymvpn.manager.backend.BackendManager
import net.nymtech.vpn.backend.Tunnel
import javax.inject.Inject

@AndroidEntryPoint
class BootReceiver : BroadcastReceiver() {

	@Inject
	lateinit var settingsRepository: SettingsRepository

	@Inject
	lateinit var backendManager: BackendManager

	@Inject
	@ApplicationScope
	lateinit var applicationScope: CoroutineScope

	override fun onReceive(context: Context, intent: Intent) {
		if (Intent.ACTION_BOOT_COMPLETED != intent.action) return
		applicationScope.launch {
			if (settingsRepository.isAutoStartEnabled()) {
				if (backendManager.getState() != Tunnel.State.Down) return@launch
				backendManager.startTunnel()
			}
		}
	}
}
