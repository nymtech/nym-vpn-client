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
import timber.log.Timber
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
		Timber.w("BootReceiver.onReceive action=${intent.action} extras=${intent.extras?.keySet()}")

		val action = intent.action ?: return
		val isBootAction =
			action == Intent.ACTION_BOOT_COMPLETED ||
				action == Intent.ACTION_LOCKED_BOOT_COMPLETED

		if (!isBootAction) return
		val pendingResult = goAsync()

		applicationScope.launch {
			try {
				val enabled = settingsRepository.isAutoStartEnabled()
				Timber.w("BootReceiver: autoStartEnabled=$enabled")
				if (!enabled) return@launch
				val state = backendManager.getState()
				Timber.w("BootReceiver: currentTunnelState=$state")
				if (state != Tunnel.State.Down) return@launch
				Timber.w("BootReceiver: starting tunnel")
				backendManager.startTunnel()
			} catch (t: Throwable) {
				Timber.e(t, "BootReceiver: failed to autostart tunnel")
			} finally {
				pendingResult.finish()
			}
		}
	}
}
