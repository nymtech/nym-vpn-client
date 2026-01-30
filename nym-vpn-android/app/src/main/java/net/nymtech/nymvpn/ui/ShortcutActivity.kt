package net.nymtech.nymvpn.ui

import android.os.Bundle
import android.widget.Toast
import androidx.biometric.BiometricPrompt
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
import net.nymtech.nymvpn.manager.shortcut.ShortcutAction
import net.nymtech.nymvpn.util.DeviceAuthHelper
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

	override fun onCreate(savedInstanceState: Bundle?) {
		super.onCreate(savedInstanceState)

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

			if (!DeviceAuthHelper.isDeviceSecure(this@ShortcutActivity)) {
				Toast.makeText(
					this@ShortcutActivity,
					getString(R.string.shortcuts_info_message),
					Toast.LENGTH_SHORT,
				).show()
				finish()
				return@launch
			}

			val promptInfo = buildShortcutPromptInfo(action)

			DeviceAuthHelper.authenticate(
				activity = this@ShortcutActivity,
				promptInfo = promptInfo,
				onAuthenticated = {
					applicationScope.launch {
						performAction(action)
					}
					finish()
				},
				onUnavailable = {
					Toast.makeText(
						this@ShortcutActivity,
						getString(R.string.shortcuts_info_message),
						Toast.LENGTH_SHORT,
					).show()
					finish()
				},
				onError = { _, _ ->
					finish()
				},
			)
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

	@Suppress("DEPRECATION")
	private fun buildShortcutPromptInfo(action: ShortcutAction): BiometricPrompt.PromptInfo {
		val title = getString(R.string.shortcut_title)
		val subtitle = when (action) {
			ShortcutAction.STOP -> getString(R.string.shortcut_subtitle_stop)
			ShortcutAction.START_MIXNET -> getString(R.string.shortcut_subtitle_start_mixnet)
			ShortcutAction.START_WG -> getString(R.string.shortcut_subtitle_start_wg)
		}

		return DeviceAuthHelper.buildPromptInfo(
			context = this,
			title = title,
			subtitle = subtitle,
		)
	}
}
