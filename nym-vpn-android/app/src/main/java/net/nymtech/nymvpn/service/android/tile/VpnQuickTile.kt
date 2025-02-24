package net.nymtech.nymvpn.service.android.tile

import android.os.Build
import android.service.quicksettings.Tile
import android.service.quicksettings.TileService
import androidx.lifecycle.Lifecycle
import androidx.lifecycle.LifecycleOwner
import androidx.lifecycle.LifecycleRegistry
import androidx.lifecycle.lifecycleScope
import dagger.hilt.android.AndroidEntryPoint
import kotlinx.coroutines.flow.catch
import kotlinx.coroutines.launch
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.data.SettingsRepository
import net.nymtech.nymvpn.manager.backend.BackendManager
import net.nymtech.nymvpn.util.extensions.toDisplayCountry
import net.nymtech.nymvpn.util.extensions.truncateWithEllipsis
import net.nymtech.vpn.backend.Tunnel
import nym_vpn_lib.EntryPoint
import nym_vpn_lib.ExitPoint
import timber.log.Timber
import java.util.*
import javax.inject.Inject

@AndroidEntryPoint
class VpnQuickTile : TileService(), LifecycleOwner {

	@Inject
	lateinit var settingsRepository: SettingsRepository

	@Inject
	lateinit var backendManager: BackendManager

	private val lifecycleRegistry: LifecycleRegistry = LifecycleRegistry(this)
	private var isCollecting = false

	override fun onCreate() {
		super.onCreate()
		lifecycleRegistry.handleLifecycleEvent(Lifecycle.Event.ON_CREATE)
	}

	override fun onStartListening() {
		super.onStartListening()
		lifecycleRegistry.handleLifecycleEvent(Lifecycle.Event.ON_START)

		if (isCollecting) return
		isCollecting = true

		lifecycleScope.launch {
			if (!backendManager.isMnemonicStored()) return@launch setUnavailable()

			backendManager.stateFlow.catch { error ->
				Timber.e(error, "Error collecting VPN state flow in tile")
				setUnavailable()
			}.collect {
				updateTileForState(it.tunnelState)
			}
		}
	}

	private suspend fun updateTileForState(state: Tunnel.State) {
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
			Tunnel.State.InitializingClient -> {
				setTileDescription(this@VpnQuickTile.getString(R.string.initializing))
				setActive()
			}
			Tunnel.State.EstablishingConnection -> {
				setTileDescription(this@VpnQuickTile.getString(R.string.connecting))
				setActive()
			}

			Tunnel.State.Offline -> {
				setTileDescription(this@VpnQuickTile.getString(R.string.offline))
				setActive()
			}
		}
	}

	override fun onTileAdded() {
		super.onTileAdded()
		onStartListening()
	}

	override fun onStopListening() {
		lifecycleRegistry.handleLifecycleEvent(Lifecycle.Event.ON_STOP)
		isCollecting = false
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
					Tunnel.State.Down -> backendManager.startTunnel()
					else -> backendManager.stopTunnel()
				}
			}
		}
	}

	private suspend fun setTileText() {
		kotlin.runCatching {
			val entryPoint = settingsRepository.getEntryPoint()
			val exitPoint = settingsRepository.getExitPoint()
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
			// TODO improve to use country code of individual nodes
			val entryText = when (entryPoint) {
				is EntryPoint.Gateway -> entryPoint.identity.truncateWithEllipsis(3)
				is EntryPoint.Location -> entryPoint.toDisplayCountry()
				else -> this@VpnQuickTile.getString(R.string.unknown)
			}

			val exitText = when (exitPoint) {
				is ExitPoint.Gateway -> exitPoint.identity.truncateWithEllipsis(3)
				is ExitPoint.Location -> exitPoint.toDisplayCountry()
				is ExitPoint.Address -> exitPoint.address.truncateWithEllipsis(3)
			}

			setTileDescription(
				"$entryText -> $exitText",
			)
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
