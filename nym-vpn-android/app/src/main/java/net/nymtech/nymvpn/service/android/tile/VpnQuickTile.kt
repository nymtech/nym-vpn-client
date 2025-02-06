package net.nymtech.nymvpn.service.android.tile

import android.os.Build
import android.service.quicksettings.Tile
import android.service.quicksettings.TileService
import androidx.lifecycle.Lifecycle
import androidx.lifecycle.LifecycleOwner
import androidx.lifecycle.LifecycleRegistry
import androidx.lifecycle.lifecycleScope
import dagger.hilt.android.AndroidEntryPoint
import kotlinx.coroutines.launch
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.data.SettingsRepository
import net.nymtech.nymvpn.manager.backend.BackendManager
import net.nymtech.vpn.backend.Tunnel
import timber.log.Timber
import javax.inject.Inject

@AndroidEntryPoint
class VpnQuickTile : TileService(), LifecycleOwner {

	@Inject
	lateinit var settingsRepository: SettingsRepository

	@Inject
	lateinit var backendManager: BackendManager

	private val lifecycleRegistry: LifecycleRegistry = LifecycleRegistry(this)

	override fun onCreate() {
		super.onCreate()
		lifecycleRegistry.handleLifecycleEvent(Lifecycle.Event.ON_CREATE)
	}

	override fun onStartListening() {
		super.onStartListening()
		lifecycleRegistry.handleLifecycleEvent(Lifecycle.Event.ON_START)

		lifecycleScope.launch {
			if (!backendManager.isMnemonicStored()) return@launch setUnavailable()
			val state = backendManager.getState()
			kotlin.runCatching {
				when (state) {
					Tunnel.State.Up -> {
						setTileText()
						setActive()
					}
					// TODO once we get offline designs, change this
					Tunnel.State.Down, Tunnel.State.Offline -> {
						setTileText()
						setInactive()
					}
					Tunnel.State.Disconnecting -> {
						setTileDescription(this@VpnQuickTile.getString(R.string.disconnecting))
						setActive()
					}
					Tunnel.State.InitializingClient -> {
						setTileDescription(this@VpnQuickTile.getString(R.string.initializing))
						setInactive()
					}
					Tunnel.State.EstablishingConnection -> {
						setTileDescription(this@VpnQuickTile.getString(R.string.connecting))
						setInactive()
					}
				}
			}.onFailure {
				Timber.e(it)
			}
		}
	}

	override fun onTileAdded() {
		super.onTileAdded()
		onStartListening()
	}

	override fun onStopListening() {
		lifecycleRegistry.handleLifecycleEvent(Lifecycle.Event.ON_STOP)
	}

	override fun onDestroy() {
		super.onDestroy()
		lifecycleRegistry.handleLifecycleEvent(Lifecycle.Event.ON_DESTROY)
	}

	override fun onClick() {
		super.onClick()
		unlockAndRun {
			lifecycleScope.launch {
				when (backendManager.getState()) {
					Tunnel.State.Up -> backendManager.stopTunnel()
					Tunnel.State.Down -> backendManager.startTunnel()
					else -> Unit
				}
			}
		}
	}

	private suspend fun setTileText() {
		kotlin.runCatching {
			// TODO fix
			val firstHopCountry = settingsRepository.getEntryPoint()
			val lastHopCountry = settingsRepository.getExitPoint()
			val mode = settingsRepository.getVpnMode()
			val isTwoHop = mode == Tunnel.Mode.TWO_HOP_MIXNET
			setTitle(
				"${this@VpnQuickTile.getString(R.string.mode)}: ${
					if (isTwoHop) {
						this@VpnQuickTile.getString(
							R.string.two_hop,
						)
					} else {
						this@VpnQuickTile.getString(R.string.five_hop)
					}
				}",
			)
// 			setTileDescription(
// 				"${firstHopCountry.isoCode} -> ${lastHopCountry.isoCode}",
// 			)
			qsTile.updateTile()
		}
	}

	private fun setActive() {
		kotlin.runCatching {
			qsTile.state = Tile.STATE_ACTIVE
			qsTile.updateTile()
		}
	}

	private fun setTitle(title: String) {
		kotlin.runCatching {
			qsTile.label = title
			qsTile.updateTile()
		}
	}

	private fun setInactive() {
		kotlin.runCatching {
			qsTile.state = Tile.STATE_INACTIVE
			qsTile.updateTile()
		}
	}

	private fun setUnavailable() {
		kotlin.runCatching {
			qsTile.state = Tile.STATE_UNAVAILABLE
			qsTile.updateTile()
		}
	}

	private fun setTileDescription(description: String) {
		kotlin.runCatching {
			if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q) {
				qsTile.subtitle = description
			}
			if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.R) {
				qsTile.stateDescription = description
			}
			qsTile.updateTile()
		}
	}

	override val lifecycle: Lifecycle
		get() = lifecycleRegistry
}
