package net.nymtech.nymvpn.ui

import android.os.Bundle
import androidx.activity.ComponentActivity
import dagger.hilt.android.AndroidEntryPoint
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.launch
import net.nymtech.nymvpn.data.SettingsRepository
import net.nymtech.nymvpn.manager.shortcut.ShortcutAction
import net.nymtech.nymvpn.module.qualifiers.ApplicationScope
import net.nymtech.nymvpn.service.tunnel.TunnelManager
import net.nymtech.vpn.backend.Tunnel
import timber.log.Timber
import javax.inject.Inject

@AndroidEntryPoint
class ShortcutActivity : ComponentActivity() {

	@Inject
	lateinit var settingsRepository: SettingsRepository

	@Inject
	@ApplicationScope
	lateinit var applicationScope: CoroutineScope

	@Inject
	lateinit var tunnelManager: TunnelManager

	override fun onCreate(savedInstanceState: Bundle?) {
		super.onCreate(savedInstanceState)
		applicationScope.launch {
			if (settingsRepository.isApplicationShortcutsEnabled()) {
				when (intent.action) {
					ShortcutAction.START_MIXNET.name -> {
						settingsRepository.setVpnMode(Tunnel.Mode.FIVE_HOP_MIXNET)
						tunnelManager.start(true)
					}
					ShortcutAction.START_WG.name -> {
						settingsRepository.setVpnMode(Tunnel.Mode.TWO_HOP_MIXNET)
						tunnelManager.start(true)
					}
					ShortcutAction.STOP.name -> tunnelManager.stop()
				}
			} else {
				Timber.w("Shortcuts not enabled")
			}
		}
		finish()
	}
}
