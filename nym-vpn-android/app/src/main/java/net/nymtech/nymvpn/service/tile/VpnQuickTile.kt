package net.nymtech.nymvpn.service.tile

import android.os.Build
import android.service.quicksettings.Tile
import android.service.quicksettings.TileService
import dagger.hilt.android.AndroidEntryPoint
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Job
import kotlinx.coroutines.cancel
import kotlinx.coroutines.launch
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.data.SettingsRepository
import net.nymtech.nymvpn.module.ApplicationScope
import net.nymtech.nymvpn.module.ServiceScope
import net.nymtech.nymvpn.service.vpn.VpnManager
import net.nymtech.vpn.VpnClient
import net.nymtech.vpn.model.VpnMode
import net.nymtech.vpn.model.VpnState
import timber.log.Timber
import javax.inject.Inject
import javax.inject.Provider

@AndroidEntryPoint
class VpnQuickTile : TileService() {

	@Inject
	lateinit var settingsRepository: SettingsRepository

	@Inject
	lateinit var vpnManager: VpnManager

	@Inject
	lateinit var vpnClient: Provider<VpnClient>

	@Inject
	@ServiceScope
	lateinit var serviceScope: CoroutineScope

	@Inject
	@ApplicationScope
	lateinit var applicationScope: CoroutineScope

	private var job: Job? = null

	override fun onStartListening() {
		super.onStartListening()
		Timber.d("Quick tile listening called")
		setTileText()
		val state = vpnClient.get().getState()
		when (state.vpnState) {
			VpnState.Up -> {
				setActive()
				setTileText()
				qsTile.updateTile()
			}

			VpnState.Down -> {
				setInactive()
				setTileText()
				qsTile.updateTile()
			}

			VpnState.Connecting.EstablishingConnection, VpnState.Connecting.InitializingClient -> {
				setTileDescription(this@VpnQuickTile.getString(R.string.connecting))
				qsTile.updateTile()
			}

			VpnState.Disconnecting -> {
				setTileDescription(this@VpnQuickTile.getString(R.string.disconnecting))
				qsTile.updateTile()
			}
		}
	}

	override fun onTileAdded() {
		super.onTileAdded()
		onStartListening()
	}

	override fun onClick() {
		super.onClick()
		setTileText()
		unlockAndRun {
			when (vpnClient.get().getState().vpnState) {
				VpnState.Up -> {
					applicationScope.launch {
						setTileDescription(this@VpnQuickTile.getString(R.string.disconnecting))
						qsTile.updateTile()
						vpnClient.get().stop(this@VpnQuickTile, true)
						job = updateOnState(VpnState.Down)
					}
				}
				VpnState.Down -> {
					applicationScope.launch {
						vpnManager.startVpn(true).onFailure {
							// TODO handle failure
							Timber.w(it)
						}.onSuccess {
							job = updateOnState(VpnState.Up)
						}
					}
				}
				else -> Unit
			}
		}
	}

	private suspend fun updateOnState(vpnState: VpnState) = serviceScope.launch {
		vpnClient.get().stateFlow.collect {
			if (it.vpnState == vpnState) {
				onStartListening()
				job?.cancel()
			}
		}
	}

	private fun setTileText() = serviceScope.launch {
		val firstHopCountry = settingsRepository.getFirstHopCountry()
		val lastHopCountry = settingsRepository.getLastHopCountry()
		val mode = settingsRepository.getVpnMode()
		val isTwoHop = mode == VpnMode.TWO_HOP_MIXNET
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

	private fun setActive() {
		qsTile.state = Tile.STATE_ACTIVE
	}

	private fun setTitle(title: String) {
		qsTile.label = title
	}

	private fun setInactive() {
		qsTile.state = Tile.STATE_INACTIVE
	}

	private fun setUnavailable() {
		qsTile.state = Tile.STATE_UNAVAILABLE
	}

	private fun setTileDescription(description: String) {
		if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q) {
			qsTile.subtitle = description
		}
		if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.R) {
			qsTile.stateDescription = description
		}
	}
}
