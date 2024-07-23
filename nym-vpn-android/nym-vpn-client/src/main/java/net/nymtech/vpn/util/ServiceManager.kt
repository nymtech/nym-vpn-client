package net.nymtech.vpn.util

import android.app.ForegroundServiceTypeException
import android.app.Service
import android.content.Context
import android.content.Intent
import android.os.Build
import net.nymtech.vpn.NymVpnService
import timber.log.Timber

object ServiceManager {
	private fun <T : Service> actionOnService(action: Action, context: Context, cls: Class<T>, extras: Map<String, String>? = null) {
		val intent =
			Intent(context, cls).also {
				it.action = action.name
				extras?.forEach { (k, v) -> it.putExtra(k, v) }
			}
		intent.component?.javaClass
		try {
			when (action) {
				Action.START_FOREGROUND, Action.STOP_FOREGROUND ->
					startForeground(context, intent)
				Action.START, Action.STOP -> context.startService(intent)
			}
		} catch (e: SecurityException) {
			Timber.e(e)
		} catch (e: IllegalStateException) {
			Timber.e(e)
		}
	}

	private fun startForeground(context: Context, intent: Intent) {
		if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
			if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.UPSIDE_DOWN_CAKE) {
				try {
					context.startForegroundService(intent)
				} catch (e: ForegroundServiceTypeException) {
					Timber.e(e)
				}
			} else {
				context.startForegroundService(intent)
			}
		} else {
			context.startService(intent)
		}
	}

	fun startVpnService(context: Context, extras: Map<String, String>? = null) {
		Timber.d("Called start vpn service")
		actionOnService(
			Action.START,
			context,
			NymVpnService::class.java,
			extras = extras,
		)
	}

	fun startVpnServiceForeground(context: Context, extras: Map<String, String>? = null) {
		Timber.d("Called start vpn service foreground")
		actionOnService(
			Action.START_FOREGROUND,
			context,
			NymVpnService::class.java,
			extras = extras,
		)
	}

	fun stopVpnServiceForeground(context: Context) {
		actionOnService(
			Action.STOP_FOREGROUND,
			context,
			NymVpnService::class.java,
		)
	}

	fun stopVpnService(context: Context) {
		actionOnService(
			Action.STOP,
			context,
			NymVpnService::class.java,
		)
	}
}
