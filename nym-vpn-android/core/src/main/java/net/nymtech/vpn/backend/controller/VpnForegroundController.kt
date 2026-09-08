package net.nymtech.vpn.backend.controller

import android.app.Service
import android.content.pm.ServiceInfo
import android.os.Build
import androidx.core.app.NotificationManagerCompat
import net.nymtech.vpn.backend.Tunnel
import net.nymtech.vpn.backend.service.VpnService
import net.nymtech.vpn.util.notifications.VpnNotificationManager
import nym_vpn_lib_types.EntryPoint
import nym_vpn_lib_types.ExitPoint
import timber.log.Timber

/**
 * Foreground & notification management.
 */
class VpnForegroundController(private val service: VpnService) {
	companion object {
		private const val TAG = "core-vpn"
	}

	fun promoteMinimal(source: String) {
		try {
			val nm = VpnNotificationManager.getInstance(service)
			val notification = nm.buildMinimalNotification()

			if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q) {
				val type =
					if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.UPSIDE_DOWN_CAKE) {
						ServiceInfo.FOREGROUND_SERVICE_TYPE_SYSTEM_EXEMPTED
					} else {
						0
					}
				service.startForeground(VpnNotificationManager.VPN_FOREGROUND_ID, notification, type)
			} else {
				service.startForeground(VpnNotificationManager.VPN_FOREGROUND_ID, notification)
			}

			Timber.tag(TAG).d("ForegroundPromoted source=%s", source)
		} catch (t: Throwable) {
			Timber.tag(TAG).e(t, "ForegroundPromoteFailed source=%s", source)
		}
	}

	fun stopForegroundSafely() {
		runCatching { service.stopForeground(Service.STOP_FOREGROUND_REMOVE) }
	}

	fun cancelForegroundNotificationSafely() {
		runCatching {
			val nm = VpnNotificationManager.getInstance(service)
			nm.withNotificationPermission {
				NotificationManagerCompat.from(service)
					.cancel(VpnNotificationManager.VPN_FOREGROUND_ID)
			}
		}
	}

	fun updateForegroundNotification(state: Tunnel.State, entry: EntryPoint?, exit: ExitPoint?, retryAttempt: UInt? = null) {
		val nm = VpnNotificationManager.getInstance(service)
		nm.updateVpnNotification(
			state = state,
			entry = entry,
			exit = exit,
			gatewaysEntry = null,
			gatewaysExit = null,
			retryAttempt = retryAttempt,
		)
	}
}
