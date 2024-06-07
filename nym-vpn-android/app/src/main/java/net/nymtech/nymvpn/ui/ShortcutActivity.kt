package net.nymtech.nymvpn.ui

import android.os.Bundle
import androidx.activity.ComponentActivity
import dagger.hilt.android.AndroidEntryPoint
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import net.nymtech.nymvpn.data.SettingsRepository
import net.nymtech.nymvpn.module.ApplicationScope
import net.nymtech.nymvpn.module.IoDispatcher
import net.nymtech.nymvpn.service.vpn.VpnManager
import net.nymtech.vpn.util.Action
import timber.log.Timber
import javax.inject.Inject

@AndroidEntryPoint
class ShortcutActivity : ComponentActivity() {

	@Inject
	lateinit var vpnManager: VpnManager

	@Inject
	lateinit var settingsRepository: SettingsRepository

	@Inject
	@ApplicationScope
	lateinit var applicationScope: CoroutineScope

	@Inject
	@IoDispatcher
	lateinit var ioDispatcher: CoroutineDispatcher

	override fun onCreate(savedInstanceState: Bundle?) {
		super.onCreate(savedInstanceState)
		applicationScope.launch {
			val enabled = withContext(ioDispatcher) {
				settingsRepository.isApplicationShortcutsEnabled()
			}
			if (enabled) {
				when (intent.action) {
					Action.START.name -> {
						vpnManager.startVpn(true).onFailure {
							Timber.w(it)
						}
					}
					Action.STOP.name -> {
						vpnManager.stopVpn(true)
					}
				}
			} else {
				Timber.w("Shortcuts not enabled")
			}
		}
		finish()
	}
}
