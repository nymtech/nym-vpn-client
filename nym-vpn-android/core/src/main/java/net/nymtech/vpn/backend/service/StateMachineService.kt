package net.nymtech.vpn.backend.service

import android.content.Intent
import android.os.PowerManager
import androidx.core.app.ServiceCompat
import androidx.lifecycle.LifecycleService
import kotlinx.coroutines.CompletableDeferred
import net.nymtech.vpn.backend.NymBackend
import net.nymtech.vpn.backend.NymBackend.Companion.stateMachineService
import net.nymtech.vpn.util.notifications.VpnNotificationManager
import timber.log.Timber

// Separate service to keep app/state machine active even after tunnel shutdown so we can keep retrying new connections
internal class StateMachineService : LifecycleService(), TunnelOwner {

	private val notificationManager = VpnNotificationManager.getInstance(this)

	override var owner: NymBackend? = null
	private var wakeLock: PowerManager.WakeLock? = null

	companion object {
		const val SYSTEM_EXEMPT_SERVICE_TYPE_ID = 1024
		const val FOREGROUND_NOTIFICATION_ID = VpnNotificationManager.VPN_FOREGROUND_ID
	}

	override fun onCreate() {
		super.onCreate()
		stateMachineService.complete(this)

		// Immediately start foreground with minimal notification to satisfy Android
		startForeground(FOREGROUND_NOTIFICATION_ID, notificationManager.buildMinimalNotification())

		// Then update with the richer notification if permission granted
		notificationManager.withNotificationPermission {
			val richNotification = notificationManager.buildVpnNotification(getCurrentState(), getCurrentEntryPoint(), getCurrentExitPoint(), getEntryGateways(), getExitGateways())
			ServiceCompat.startForeground(
				this,
				FOREGROUND_NOTIFICATION_ID,
				richNotification,
				SYSTEM_EXEMPT_SERVICE_TYPE_ID,
			)
		}

		initWakeLock()
	}

	override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
		stateMachineService.complete(this)

		// Same here: call startForeground synchronously with minimal notification first
		startForeground(FOREGROUND_NOTIFICATION_ID, notificationManager.buildMinimalNotification())

		// Then update notification if permission granted
		notificationManager.withNotificationPermission {
			val richNotification = notificationManager.buildVpnNotification(getCurrentState(), getCurrentEntryPoint(), getCurrentExitPoint(), getEntryGateways(), getExitGateways())
			ServiceCompat.startForeground(
				this,
				FOREGROUND_NOTIFICATION_ID,
				richNotification,
				SYSTEM_EXEMPT_SERVICE_TYPE_ID,
			)
		}

		return super.onStartCommand(intent, flags, startId)
	}

	override fun onDestroy() {
		stateMachineService = CompletableDeferred()
		wakeLock?.let {
			if (it.isHeld) {
				it.release()
			}
		}
		ServiceCompat.stopForeground(this, ServiceCompat.STOP_FOREGROUND_REMOVE)
		super.onDestroy()
	}

	private fun initWakeLock() {
		wakeLock = (getSystemService(POWER_SERVICE) as PowerManager).run {
			val tag = this.javaClass.name
			newWakeLock(PowerManager.PARTIAL_WAKE_LOCK, "$tag::lock").apply {
				try {
					Timber.d("Initiating wakelock")
					acquire()
				} finally {
					release()
				}
			}
		}
	}
}
