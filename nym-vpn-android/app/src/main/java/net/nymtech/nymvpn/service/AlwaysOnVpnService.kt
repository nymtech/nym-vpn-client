package net.nymtech.nymvpn.service

import android.app.Service
import android.content.Intent
import android.os.IBinder
import dagger.hilt.android.AndroidEntryPoint
import net.nymtech.nymvpn.NymVpn
import net.nymtech.nymvpn.data.datastore.DataStoreManager
import net.nymtech.vpn.NymVpnClient
import net.nymtech.vpn.model.Hop
import net.nymtech.vpn.model.VpnMode
import timber.log.Timber
import javax.inject.Inject

@AndroidEntryPoint
class AlwaysOnVpnService : Service() {

    @Inject
    lateinit var dataStoreManager: DataStoreManager
    override fun onBind(intent: Intent?): IBinder? {
        return null
    }

    override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
        if (intent == null || intent.component == null || intent.component!!.packageName != packageName) {
            Timber.i("Always-on VPN requested start")
            val firstHopCountry = dataStoreManager.getFromStoreBlocking(DataStoreManager.FIRST_HOP_COUNTRY)
            val lastHopCountry = dataStoreManager.getFromStoreBlocking(DataStoreManager.LAST_HOP_COUNTRY)
            val mode = dataStoreManager.getFromStoreBlocking(DataStoreManager.VPN_MODE)
            NymVpn.requestTileServiceStateUpdate(this)
            NymVpnClient.connectForeground(this, Hop.Country.from(firstHopCountry), Hop.Country.from(lastHopCountry),
                VpnMode.from(mode))
            START_STICKY
        } else {
            START_NOT_STICKY
        }
        return super.onStartCommand(intent, flags, startId)
    }
}