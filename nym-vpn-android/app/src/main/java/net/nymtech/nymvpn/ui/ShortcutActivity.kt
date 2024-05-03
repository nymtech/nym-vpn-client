package net.nymtech.nymvpn.ui

import android.os.Bundle
import androidx.activity.ComponentActivity
import dagger.hilt.android.AndroidEntryPoint
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import net.nymtech.nymvpn.NymVpn
import net.nymtech.nymvpn.data.SettingsRepository
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

	override fun onCreate(savedInstanceState: Bundle?) {
		super.onCreate(savedInstanceState)
		NymVpn.applicationScope.launch(Dispatchers.IO) {
			if (settingsRepository.isApplicationShortcutsEnabled()) {
				when (intent.action) {
					Action.START.name -> {
						vpnManager.startVpn(this@ShortcutActivity, true).onFailure {
							Timber.w(it)
						}
					}
					Action.STOP.name -> {
						vpnManager.stopVpn(this@ShortcutActivity, true)
					}
				}
			}
		}
		finish()
	}
}
