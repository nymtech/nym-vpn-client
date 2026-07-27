package net.nymtech.nymvpn.receiver

import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent
import dagger.hilt.android.AndroidEntryPoint
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.launch
import kotlinx.coroutines.withTimeoutOrNull
import net.nymtech.nymvpn.data.SettingsRepository
import net.nymtech.nymvpn.di.qualifiers.ApplicationScope
import net.nymtech.nymvpn.manager.backend.BackendManager
import net.nymtech.nymvpn.util.Constants
import net.nymtech.vpn.backend.Tunnel
import timber.log.Timber
import javax.inject.Inject
import kotlin.time.Duration.Companion.milliseconds

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

				val initializedState = withTimeoutOrNull(Constants.AUTO_START_INIT_WAIT_MS.milliseconds) {
					backendManager.stateFlow.first { it.isInitialized }
				}
				if (initializedState == null) {
					Timber.tag(TAG).w("BootAutoStartSkipped reason=backend_init_timeout")
					return@launch
				}

				val state = backendManager.getState()
				if (state == Tunnel.State.Down) {
					Timber.tag(TAG).i("BootAutoStartRequested")
					backendManager.startTunnel()
				} else {
					Timber.tag(TAG).i("BootAutoStartWatching state=%s", state)
				}
				watchAndRetryIfStuck()
			} catch (t: Throwable) {
				Timber.tag(TAG).e(t, "BootAutoStartFailed")
			} finally {
				pendingResult.finish()
			}
		}
	}

	private fun watchAndRetryIfStuck() {
		applicationScope.launch {
			runCatching {
				val reachedUp = withTimeoutOrNull(Constants.AUTO_START_STUCK_STATE_TIMEOUT_MS.milliseconds) {
					backendManager.stateFlow.map { it.tunnelState }.first { it == Tunnel.State.Up }
				}
				if (reachedUp == null) {
					Timber.tag(TAG).w("BootAutoStartStuckRetrying state=%s", backendManager.getState())
					backendManager.startTunnel()
				} else {
					Timber.tag(TAG).i("BootAutoStartConfirmedUp")
				}
			}.onFailure { t ->
				Timber.tag(TAG).e(t, "BootAutoStartRetryFailed")
			}
		}
	}
}
