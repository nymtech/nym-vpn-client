package net.nymtech.vpn.util.extensions

import android.app.Service
import android.content.Context
import android.content.Intent
import android.os.Build
import net.nymtech.vpn.util.Action
import timber.log.Timber

fun <T : Service> Context.startServiceByClass(background: Boolean, cls: Class<T>) {
	runCatching {
		val intent = Intent(this, cls).apply {
			if (background) action = Action.START_FOREGROUND.name
		}
		if (background && Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
			startForegroundService(intent)
		} else {
			startService(intent)
		}
	}.onFailure { Timber.w("Ignoring not started in time exception") }
}
