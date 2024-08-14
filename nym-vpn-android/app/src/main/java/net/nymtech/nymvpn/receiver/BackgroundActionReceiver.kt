package net.nymtech.nymvpn.receiver

import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent
import dagger.hilt.android.AndroidEntryPoint
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.launch
import net.nymtech.nymvpn.module.ApplicationScope
import net.nymtech.nymvpn.service.tunnel.TunnelManager
import javax.inject.Inject

@AndroidEntryPoint
class BackgroundActionReceiver : BroadcastReceiver() {

	@Inject
	@ApplicationScope
	lateinit var applicationScope: CoroutineScope

	@Inject
	lateinit var tunnelManager: TunnelManager

	override fun onReceive(context: Context, intent: Intent) {
		val action = intent.action ?: return
		when (action) {
			ACTION_CONNECT -> {
				applicationScope.launch {
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
