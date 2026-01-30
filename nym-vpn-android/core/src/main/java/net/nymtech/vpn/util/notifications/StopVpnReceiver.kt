package net.nymtech.vpn.util.notifications

import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent
import android.os.Build
import android.widget.Toast
import net.nymtech.vpn.backend.service.VpnService

/**
 * Broadcast receiver for VPN disconnect action.
 * Triggers VpnService to stop the tunnel.
 */
class StopVpnReceiver : BroadcastReceiver() {

	companion object {
		const val ACTION_DISCONNECT = "net.nymtech.vpn.action.DISCONNECT"
	}

	override fun onReceive(context: Context, intent: Intent?) {
		if (intent?.action != ACTION_DISCONNECT) return

		val i = Intent(context, VpnService::class.java).apply {
			action = ACTION_DISCONNECT
		}

		if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
			runCatching { context.startForegroundService(i) }
		} else {
			runCatching { context.startService(i) }
		}

		Toast.makeText(context, "VPN disconnect requested", Toast.LENGTH_SHORT).show()
	}
}
