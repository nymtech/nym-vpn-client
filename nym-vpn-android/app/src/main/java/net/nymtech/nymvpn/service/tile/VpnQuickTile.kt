package net.nymtech.nymvpn.service.tile

import android.os.Build
import android.service.quicksettings.Tile
import android.service.quicksettings.TileService
import dagger.hilt.android.AndroidEntryPoint
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.cancel
import kotlinx.coroutines.launch
import net.nymtech.nymvpn.NymVpn
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.data.GatewayRepository
import net.nymtech.nymvpn.data.SettingsRepository
import net.nymtech.vpn.NymVpnClient
import net.nymtech.vpn.model.VpnMode
import net.nymtech.vpn.model.VpnState
import timber.log.Timber
import javax.inject.Inject

@AndroidEntryPoint
class VpnQuickTile : TileService() {

    @Inject
    lateinit var gatewayRepository: GatewayRepository

    lateinit var settingsRepository: SettingsRepository

    private val scope = CoroutineScope(Dispatchers.IO)

    override fun onStartListening() {
        super.onStartListening()
        Timber.d("Quick tile listening called")
        setTileText()
        scope.launch {
            NymVpnClient.stateFlow.collect {
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

    override fun onTileAdded() {
        super.onTileAdded()
        setTileText()
    }

    override fun onDestroy() {
        super.onDestroy()
        scope.cancel()
    }

    override fun onTileRemoved() {
        super.onTileRemoved()
        scope.cancel()
    }

    override fun onClick() {
        super.onClick()
        setTileText()
        Timber.i("Tile clicked")
        unlockAndRun {
            when (NymVpnClient.getState().vpnState) {
                VpnState.Up -> NymVpnClient.disconnect(this)
                VpnState.Down -> {
                    scope.launch {
                        val entryCountry = gatewayRepository.getFirstHopCountry()
                        val exitCountry = gatewayRepository.getLastHopCountry()
                        val mode = settingsRepository.getVpnMode()
                        val entry = entryCountry.toEntryPoint()
                        val exit = exitCountry.toExitPoint()
                        NymVpnClient.configure(entry, exit, mode)
                        NymVpnClient.start(this@VpnQuickTile)
                        NymVpn.requestTileServiceStateUpdate(this@VpnQuickTile)
                    }

                }

                else -> Unit
            }
        }
    }

    private fun setTileText() = scope.launch {
        val firstHopCountry = gatewayRepository.getFirstHopCountry()
        val lastHopCountry = gatewayRepository.getLastHopCountry()
        val mode = settingsRepository.getVpnMode()
        val isTwoHop = mode == VpnMode.TWO_HOP_MIXNET
        setTitle(
            "${this@VpnQuickTile.getString(R.string.mode)}: ${
                if (isTwoHop) this@VpnQuickTile.getString(
                    R.string.two_hop
                ) else this@VpnQuickTile.getString(R.string.five_hop)
            }"
        )
        setTileDescription(
            "${firstHopCountry.isoCode} -> ${lastHopCountry.isoCode}"
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