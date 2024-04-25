package net.nymtech.nymvpn.service.tile

import android.os.Build
import android.service.quicksettings.Tile
import android.service.quicksettings.TileService
import dagger.hilt.android.AndroidEntryPoint
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.cancel
import kotlinx.coroutines.launch
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.data.SecretsRepository
import net.nymtech.nymvpn.data.SettingsRepository
import net.nymtech.vpn.VpnClient
import net.nymtech.vpn.model.VpnMode
import net.nymtech.vpn.model.VpnState
import net.nymtech.vpn.util.InvalidCredentialException
import timber.log.Timber
import javax.inject.Inject

@AndroidEntryPoint
class VpnQuickTile : TileService() {
	@Inject
	lateinit var secretsRepository: SecretsRepository

	@Inject
	lateinit var settingsRepository: SettingsRepository

	@Inject
	lateinit var vpnClient: VpnClient

	private val scope = CoroutineScope(Dispatchers.Main)

	override fun onStartListening() {
		super.onStartListening()
		Timber.d("Quick tile listening called")
		setTileText()
		scope.launch {
			vpnClient.stateFlow.collect {
				when (it.vpnState) {
					VpnState.Up -> {
						setActive()
						setTileText()
					}

					VpnState.Down -> {
						setInactive()
						setTileText()
					}

					VpnState.Connecting.EstablishingConnection, VpnState.Connecting.InitializingClient -> {
						setTileDescription(this@VpnQuickTile.getString(R.string.connecting))
					}

					VpnState.Disconnecting -> {
						setTileDescription(this@VpnQuickTile.getString(R.string.disconnecting))
					}
				}
			}
		}
	}

	override fun onDestroy() {
		super.onDestroy()
		scope.cancel()
	}

	override fun onTileRemoved() {
		super.onTileRemoved()
		scope.cancel()
	}

	override fun onTileAdded() {
		super.onTileAdded()
		onStartListening()
	}

	override fun onClick() {
		super.onClick()
		setTileText()
		unlockAndRun {
			when (vpnClient.getState().vpnState) {
				VpnState.Up -> {
					scope.launch {
						setTileDescription(this@VpnQuickTile.getString(R.string.disconnecting))
						vpnClient.stop(this@VpnQuickTile, true)
					}
				}
				VpnState.Down -> {
					scope.launch {
						val credential = secretsRepository.getCredential()
						if (credential != null) {
							try {
								vpnClient.apply {
									this.mode = settingsRepository.getVpnMode()
									this.exitPoint = settingsRepository.getLastHopCountry().toExitPoint()
									this.entryPoint = settingsRepository.getFirstHopCountry().toEntryPoint()
								}.start(this@VpnQuickTile, credential, true)
							} catch (e: InvalidCredentialException) {
								Timber.w(e)
							}
						}
					}
				}
				else -> Unit
			}
		}
	}

	private fun setTileText() = scope.launch {
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
		qsTile.updateTile()
	}

	private fun setTitle(title: String) {
		qsTile.label = title
		qsTile.updateTile()
	}

	private fun setInactive() {
		qsTile.state = Tile.STATE_INACTIVE
		qsTile.updateTile()
	}

	private fun setUnavailable() {
		qsTile.state = Tile.STATE_UNAVAILABLE
		qsTile.updateTile()
	}

	private fun setTileDescription(description: String) {
		if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q) {
			qsTile.subtitle = description
		}
		if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.R) {
			qsTile.stateDescription = description
		}
		qsTile.updateTile()
	}
}
