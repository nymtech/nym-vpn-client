package net.nymtech.nymvpn.receiver

import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent
import dagger.hilt.android.AndroidEntryPoint
import net.nymtech.nymvpn.NymVpn
import net.nymtech.nymvpn.data.datastore.DataStoreManager
import net.nymtech.nymvpn.util.goAsync
import net.nymtech.vpn.NymVpnClient
import net.nymtech.vpn.model.Hop
import net.nymtech.vpn.model.VpnMode
import javax.inject.Inject

@AndroidEntryPoint
class BootReceiver : BroadcastReceiver() {

    @Inject lateinit var dataStoreManager: DataStoreManager

    override fun onReceive(context: Context?, intent: Intent?) = goAsync {
        if (Intent.ACTION_BOOT_COMPLETED != intent?.action) return@goAsync
        val autoStart = dataStoreManager.getFromStore(DataStoreManager.AUTO_START)
        if (autoStart == true) {
            val firstHopCountry = dataStoreManager.getFromStore(DataStoreManager.FIRST_HOP_COUNTRY)
            val lastHopCountry = dataStoreManager.getFromStore(DataStoreManager.LAST_HOP_COUNTRY)
            val mode = dataStoreManager.getFromStore(DataStoreManager.NETWORK_MODE)
            context?.let { context ->
                NymVpnClient.connectForeground(context, Hop.Country.from(firstHopCountry), Hop.Country.from(lastHopCountry),
                    VpnMode.from(mode))
                NymVpn.requestTileServiceStateUpdate(context)
            }
        }
    }
}