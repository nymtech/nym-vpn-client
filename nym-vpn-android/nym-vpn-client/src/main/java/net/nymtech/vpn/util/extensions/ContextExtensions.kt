package net.nymtech.vpn.util.extensions

import android.content.Context
import android.content.Intent
import android.os.Build
import net.nymtech.vpn.backend.NymBackend.VpnService
import timber.log.Timber

fun Context.startVpnService(background: Boolean) {
	runCatching {
		val intent = Intent(this, VpnService::class.java)
		if (background && Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
			startForegroundService(intent)
		} else {
			startService(intent)
		}
	}.onFailure { Timber.w("Ignoring not started in time exception") }
}
