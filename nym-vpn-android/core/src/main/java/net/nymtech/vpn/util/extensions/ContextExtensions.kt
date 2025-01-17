package net.nymtech.vpn.util.extensions

import android.app.Service
import android.content.Context
import android.content.Intent
import android.os.Build
import timber.log.Timber

fun <T : Service> Context.startServiceByClass(cls: Class<T>) {
	runCatching {
		val intent = Intent(this, cls)
		if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
			startForegroundService(intent)
		} else {
			startService(intent)
		}
	}.onFailure { Timber.w("Ignoring not started in time exception") }
}
