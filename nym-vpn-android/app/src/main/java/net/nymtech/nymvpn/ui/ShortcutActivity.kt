package net.nymtech.nymvpn.ui

import android.os.Bundle
import androidx.activity.compose.setContent
import androidx.compose.ui.res.stringResource
import androidx.fragment.app.FragmentActivity
import androidx.lifecycle.lifecycleScope
import dagger.hilt.android.AndroidEntryPoint
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.data.SettingsRepository
import net.nymtech.nymvpn.data.config.VpnConfigRepository
import net.nymtech.nymvpn.di.qualifiers.ApplicationScope
import net.nymtech.nymvpn.manager.backend.BackendManager
import net.nymtech.nymvpn.manager.deviceauth.DeviceAuthManager
import net.nymtech.nymvpn.manager.deviceauth.model.DeviceAuthResult
import net.nymtech.nymvpn.manager.shortcut.ShortcutAction
import net.nymtech.nymvpn.ui.common.modal.ScreenLockSetupDialog
import net.nymtech.nymvpn.ui.theme.NymVPNTheme
import net.nymtech.nymvpn.util.extensions.launchScreenLockSetup
import net.nymtech.vpn.backend.Tunnel
import net.nymtech.vpn.config.CoreVpnConfigUpdate
import timber.log.Timber
import javax.inject.Inject

// TODO: add dynamic shortcuts action based on tunnel state
@AndroidEntryPoint
class ShortcutActivity : FragmentActivity() {

	@Inject lateinit var settingsRepository: SettingsRepository

	@Inject lateinit var vpnConfigRepository: VpnConfigRepository

	@Inject @ApplicationScope
	lateinit var applicationScope: CoroutineScope

	@Inject lateinit var backendManager: BackendManager

	@Inject lateinit var deviceAuthManager: DeviceAuthManager

	override fun onCreate(savedInstanceState: Bundle?) {
		super.onCreate(savedInstanceState)

		if (savedInstanceState != null) {
			finish()
			return
		}

		val action = intent.action?.let { raw ->
			runCatching { ShortcutAction.valueOf(raw) }.getOrNull()
		}

		if (action == null) {
			Timber.w("ShortcutActivity: unknown/null action: ${intent.action}")
			finish()
			return
		}

		lifecycleScope.launch {
			val shortcutsEnabled = withContext(Dispatchers.IO) {
				settingsRepository.isApplicationShortcutsEnabled()
			}

			if (!shortcutsEnabled) {
				Timber.w("ShortcutActivity: shortcuts not enabled")
				finish()
				return@launch
			}

			if (!deviceAuthManager.isAuthSetUp()) {
				showScreenLockSetupDialog()
				return@launch
			}

			val result = deviceAuthManager.authenticate(
				activity = this@ShortcutActivity,
				title = getString(R.string.shortcut_title),
				subtitle = shortcutSubtitle(action),
			)
			when (result) {
				DeviceAuthResult.Success -> {
					applicationScope.launch {
						performAction(action)
					}
					finish()
				}

				DeviceAuthResult.NotSetUp -> showScreenLockSetupDialog()

				DeviceAuthResult.Cancelled, is DeviceAuthResult.Unavailable, is DeviceAuthResult.Error -> finish()
			}
		}
	}

	private suspend fun showScreenLockSetupDialog() {
		val theme = withContext(Dispatchers.IO) { settingsRepository.getTheme() }
		setContent {
			NymVPNTheme(theme = theme) {
				ScreenLockSetupDialog(
					show = true,
					body = stringResource(R.string.screen_lock_setup_shortcuts_body),
					onSetUpClick = {
						launchScreenLockSetup(deviceAuthManager.createSetupIntent())
						finish()
					},
					onDismiss = { finish() },
				)
			}
		}
	}

	private suspend fun performAction(action: ShortcutAction) {
		when (action) {
			ShortcutAction.START_MIXNET -> {
				vpnConfigRepository.apply(CoreVpnConfigUpdate.SetMode(Tunnel.Mode.FIVE_HOP_MIXNET))
				backendManager.startTunnel()
			}

			ShortcutAction.START_WG -> {
				vpnConfigRepository.apply(CoreVpnConfigUpdate.SetMode(Tunnel.Mode.TWO_HOP_MIXNET))
				backendManager.startTunnel()
			}

			ShortcutAction.STOP -> backendManager.stopTunnel()
		}
	}

	private fun shortcutSubtitle(action: ShortcutAction): String = when (action) {
		ShortcutAction.STOP -> getString(R.string.shortcut_subtitle_stop)
		ShortcutAction.START_MIXNET -> getString(R.string.shortcut_subtitle_start_mixnet)
		ShortcutAction.START_WG -> getString(R.string.shortcut_subtitle_start_wg)
	}
}
