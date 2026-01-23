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

	companion object {
		private const val TAG = "boot-receiver"
	}

	@Inject
	lateinit var settingsRepository: SettingsRepository

	@Inject
	lateinit var backendManager: BackendManager

	@Inject
	@ApplicationScope
	lateinit var applicationScope: CoroutineScope

	override fun onReceive(context: Context, intent: Intent) {
		val action = intent.action ?: return
		val isBootAction =
			action == Intent.ACTION_BOOT_COMPLETED ||
				action == Intent.ACTION_LOCKED_BOOT_COMPLETED

		if (!isBootAction) return

		Timber.tag(TAG).i("BootReceived action=%s", action)

		val pendingResult = goAsync()

		applicationScope.launch {
			try {
				val enabled = settingsRepository.isAutoStartEnabled()
				if (!enabled) {
					Timber.tag(TAG).i("BootAutoStartSkipped reason=disabled")
					return@launch
				}

				val state = backendManager.getState()
				if (state != Tunnel.State.Down) {
					Timber.tag(TAG).i("BootAutoStartSkipped reason=tunnel_not_down state=%s", state)
					return@launch
				}

				Timber.tag(TAG).i("BootAutoStartRequested")
				backendManager.startTunnel()
			} catch (t: Throwable) {
				Timber.tag(TAG).e(t, "BootAutoStartFailed")
			} finally {
				pendingResult.finish()
			}
		}
	}
}
