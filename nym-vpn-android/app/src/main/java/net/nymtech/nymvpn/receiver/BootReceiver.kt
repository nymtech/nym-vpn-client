package net.nymtech.nymvpn.receiver

import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent
import dagger.hilt.android.AndroidEntryPoint
import net.nymtech.nymvpn.data.datastore.DataStoreManager
import net.nymtech.nymvpn.util.Constants
import net.nymtech.nymvpn.util.goAsync
import net.nymtech.vpn.NymVpn
import net.nymtech.vpn.model.EntryPoint
import net.nymtech.vpn.model.ExitPoint
import net.nymtech.vpn.model.VpnMode
import javax.inject.Inject

@AndroidEntryPoint
class BootReceiver : BroadcastReceiver() {

    @Inject lateinit var dataStoreManager: DataStoreManager

    //TODO fix, this isn't working
    override fun onReceive(context: Context?, intent: Intent?) = goAsync {
        if (Intent.ACTION_BOOT_COMPLETED != intent?.action) return@goAsync
        val autoStart = dataStoreManager.getFromStore(DataStoreManager.AUTO_START)
        if (autoStart == true) {
            val entryCountryIso = dataStoreManager.getFromStore(DataStoreManager.FIRST_HOP_COUNTRY_ISO) ?: Constants.DEFAULT_COUNTRY_ISO
            val exitCountryIso = dataStoreManager.getFromStore(DataStoreManager.LAST_HOP_COUNTRY_ISO) ?: Constants.DEFAULT_COUNTRY_ISO
            val isTwoHop = dataStoreManager.getFromStore(DataStoreManager.NETWORK_MODE) == VpnMode.TWO_HOP_MIXNET.name
            context?.let {
                NymVpn.connectForeground(it,EntryPoint.Location(entryCountryIso), ExitPoint.Location(exitCountryIso),isTwoHop)
            }
        }
    }
}