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
import net.nymtech.nymvpn.service.tunnel.TunnelManager
import net.nymtech.nymvpn.util.extensions.isExpired
import net.nymtech.nymvpn.util.extensions.startTunnelFromBackground
import net.nymtech.nymvpn.util.extensions.stopTunnelFromBackground
import net.nymtech.vpn.Tunnel
import timber.log.Timber
import javax.inject.Inject

@AndroidEntryPoint
class VpnQuickTile : TileService(), LifecycleOwner {

	@Inject
	lateinit var settingsRepository: SettingsRepository

	@Inject
	lateinit var tunnelManager: TunnelManager

	private val lifecycleRegistry: LifecycleRegistry = LifecycleRegistry(this)

	override fun onCreate() {
		super.onCreate()
		lifecycleRegistry.handleLifecycleEvent(Lifecycle.Event.ON_CREATE)
	}

	override fun onStartListening() {
		super.onStartListening()
		lifecycleRegistry.handleLifecycleEvent(Lifecycle.Event.ON_START)

		lifecycleScope.launch {
			val credExpiry = settingsRepository.getCredentialExpiry()
			if (credExpiry == null || credExpiry.isExpired()) return@launch setUnavailable()
			val state = tunnelManager.getState()
			Timber.d("State from tile: $state")
			when (state) {
				Tunnel.State.Up -> {
					setTileText()
					setActive()
				}
				Tunnel.State.Down -> {
					setTileText()
					setInactive()
				}
				Tunnel.State.Disconnecting -> {
					setTileDescription(this@VpnQuickTile.getString(R.string.disconnecting))
					setActive()
				}
				Tunnel.State.Connecting.EstablishingConnection, Tunnel.State.Connecting.InitializingClient -> {
					setTileDescription(this@VpnQuickTile.getString(R.string.connecting))
					setInactive()
				}
			}
		}
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
			when (tunnelManager.getState()) {
				Tunnel.State.Up -> {
					stopTunnelFromBackground()
				}
				Tunnel.State.Down -> {
					startTunnelFromBackground()
				}
				else -> Unit
			}
		}
	}

	private fun setTileText() = lifecycleScope.launch {
		kotlin.runCatching {
			val firstHopCountry = settingsRepository.getFirstHopCountry()
			val lastHopCountry = settingsRepository.getLastHopCountry()
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
			setTileDescription(
				"${firstHopCountry.isoCode} -> ${lastHopCountry.isoCode}",
			)
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
