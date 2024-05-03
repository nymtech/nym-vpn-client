package net.nymtech.nymvpn.receiver

import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent
import dagger.hilt.android.AndroidEntryPoint
import net.nymtech.nymvpn.data.SettingsRepository
import net.nymtech.nymvpn.service.vpn.VpnManager
import net.nymtech.nymvpn.util.goAsync
import timber.log.Timber
import javax.inject.Inject

@AndroidEntryPoint
class BootReceiver : BroadcastReceiver() {

	@Inject
	lateinit var settingsRepository: SettingsRepository

	@Inject
	lateinit var vpnManager: VpnManager

	override fun onReceive(context: Context?, intent: Intent?) = goAsync {
		if (Intent.ACTION_BOOT_COMPLETED != intent?.action) return@goAsync
		if (settingsRepository.isAutoStartEnabled()) {
			context?.let { context ->
				vpnManager.startVpn(context, true).onFailure {
					// TODO handle failures
					Timber.w(it)
				}
			}
		}
	}
}
