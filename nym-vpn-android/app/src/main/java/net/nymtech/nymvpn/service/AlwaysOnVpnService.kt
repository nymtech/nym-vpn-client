package net.nymtech.nymvpn.service

import android.content.Intent
import android.os.IBinder
import androidx.lifecycle.LifecycleService
import dagger.hilt.android.AndroidEntryPoint
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.launch
import net.nymtech.nymvpn.module.ApplicationScope
import net.nymtech.nymvpn.service.vpn.VpnManager
import timber.log.Timber
import javax.inject.Inject

@AndroidEntryPoint
class AlwaysOnVpnService : LifecycleService() {

	@Inject
	lateinit var vpnManager: VpnManager

	@Inject
	@ApplicationScope
	lateinit var applicationScope: CoroutineScope

	override fun onBind(intent: Intent): IBinder? {
		super.onBind(intent)
		// We don't provide binding, so return null
		return null
	}

	override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
		if (intent == null || intent.component == null || intent.component!!.packageName != packageName) {
			Timber.i("Always-on VPN requested start")
			applicationScope.launch {
				vpnManager.startVpn(true).onFailure {
					// TODO handle failures
					Timber.w(it)
				}
			}
		}
		return super.onStartCommand(intent, flags, startId)
	}
}
