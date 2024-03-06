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
import net.nymtech.nymvpn.data.datastore.DataStoreManager
import net.nymtech.nymvpn.util.Constants
import net.nymtech.vpn.NymVpn
import net.nymtech.vpn.model.EntryPoint
import net.nymtech.vpn.model.ExitPoint
import net.nymtech.vpn.model.VpnMode
import net.nymtech.vpn.model.VpnState
import timber.log.Timber
import javax.inject.Inject

@AndroidEntryPoint
class QuickTile : TileService() {

    @Inject
    lateinit var dataStoreManager: DataStoreManager

    private val scope = CoroutineScope(Dispatchers.IO)

    override fun onStartListening() {
        super.onStartListening()
        Timber.d("Quick tile listening called")
        setTileText()
        scope.launch {
            NymVpn.stateFlow.collect {
                when(it.vpnState) {
                    VpnState.Up -> {
                        setActive()
                        setTileText()
                    }
                    VpnState.Down -> {
                        setInactive()
                        setTileText()
                    }
                    VpnState.Connecting.EstablishingConnection, VpnState.Connecting.InitializingClient -> {
                        setTileDescription(this@QuickTile.getString(R.string.connecting))
                    }
                    VpnState.Disconnecting -> {
                        setTileDescription(this@QuickTile.getString(R.string.disconnecting))
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
            when(NymVpn.getState().vpnState) {
                VpnState.Up -> NymVpn.disconnect(this)
                VpnState.Down -> {
                    scope.launch {
                        val entryCountryIso = dataStoreManager.getFromStore(DataStoreManager.FIRST_HOP_COUNTRY_ISO) ?: Constants.DEFAULT_COUNTRY_ISO
                        val exitCountryIso = dataStoreManager.getFromStore(DataStoreManager.LAST_HOP_COUNTRY_ISO) ?: Constants.DEFAULT_COUNTRY_ISO
                        val isTwoHop = dataStoreManager.getFromStore(DataStoreManager.NETWORK_MODE) == VpnMode.TWO_HOP_MIXNET.name
                        NymVpn.connect(this@QuickTile,
                            EntryPoint.Location(entryCountryIso), ExitPoint.Location(exitCountryIso),isTwoHop)
                    }
                }
                else -> Unit
            }
       }
    }

    private fun setTileText() = scope.launch {
        val entryCountryIso = dataStoreManager.getFromStore(DataStoreManager.FIRST_HOP_COUNTRY_ISO) ?: Constants.DEFAULT_COUNTRY_ISO
        val exitCountryIso = dataStoreManager.getFromStore(DataStoreManager.LAST_HOP_COUNTRY_ISO) ?: Constants.DEFAULT_COUNTRY_ISO
        val isTwoHop = dataStoreManager.getFromStore(DataStoreManager.NETWORK_MODE) == VpnMode.TWO_HOP_MIXNET.name
        setTitle("${this@QuickTile.getString(R.string.mode)}: ${if(isTwoHop) this@QuickTile.getString(
            R.string.two_hop) else this@QuickTile.getString(R.string.five_hop)}")
        setTileDescription(
            "$entryCountryIso -> $exitCountryIso")
    }

    private fun setActive() {
        qsTile.state = Tile.STATE_ACTIVE
        qsTile.updateTile()
    }

    private fun setTitle(title : String) {
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