package net.nymtech.nymvpn.receiver

import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent
import dagger.hilt.android.AndroidEntryPoint
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.launch
import net.nymtech.nymvpn.data.SettingsRepository
import net.nymtech.nymvpn.module.qualifiers.ApplicationScope
import net.nymtech.nymvpn.service.tunnel.TunnelManager
import net.nymtech.vpn.backend.Backend
import javax.inject.Inject
import javax.inject.Provider

@AndroidEntryPoint
class BackgroundActionReceiver : BroadcastReceiver() {

	@Inject
	@ApplicationScope
	lateinit var applicationScope: CoroutineScope

	@Inject
	lateinit var tunnelManager: TunnelManager

	@Inject
	lateinit var backend: Provider<Backend>

	@Inject
	lateinit var settingsRepository: SettingsRepository

	override fun onReceive(context: Context, intent: Intent) {
		val action = intent.action ?: return
		when (action) {
			ACTION_CONNECT -> {
				applicationScope.launch {
					// We need to try and init lib again because we don't know if it is running
					val env = settingsRepository.getEnvironment()
					backend.get().init(env)
					tunnelManager.start()
				}
			}
			ACTION_DISCONNECT -> {
				applicationScope.launch {
					tunnelManager.stop()
				}
			}
		}
	}

	companion object {
		const val ACTION_CONNECT = "ACTION_CONNECT"
		const val ACTION_DISCONNECT = "ACTION_DISCONNECT"
	}
}
