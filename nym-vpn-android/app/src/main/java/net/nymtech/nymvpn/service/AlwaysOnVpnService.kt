package net.nymtech.nymvpn.service

import android.content.Intent
import android.os.IBinder
import androidx.lifecycle.LifecycleService
import androidx.lifecycle.lifecycleScope
import dagger.hilt.android.AndroidEntryPoint
import kotlinx.coroutines.launch
import net.nymtech.nymvpn.NymVpn
import net.nymtech.nymvpn.data.GatewayRepository
import net.nymtech.nymvpn.data.SettingsRepository
import net.nymtech.vpn.NymVpnClient
import timber.log.Timber
import javax.inject.Inject

@AndroidEntryPoint
class AlwaysOnVpnService : LifecycleService() {

    @Inject lateinit var gatewayRepository: GatewayRepository

    @Inject lateinit var settingsRepository: SettingsRepository

    override fun onBind(intent: Intent): IBinder? {
        super.onBind(intent)
        // We don't provide binding, so return null
        return null
    }

    override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
        if (intent == null || intent.component == null || intent.component!!.packageName != packageName) {
            Timber.i("Always-on VPN requested start")
            lifecycleScope.launch {
                val entryCountry = gatewayRepository.getFirstHopCountry()
                val exitCountry = gatewayRepository.getLastHopCountry()
                val mode = settingsRepository.getVpnMode()
                val entry = entryCountry.toEntryPoint()
                val exit = exitCountry.toExitPoint()
                NymVpnClient.configure(entry,exit,mode)
                NymVpnClient.start(this@AlwaysOnVpnService)
                NymVpn.requestTileServiceStateUpdate(this@AlwaysOnVpnService)
            }
            START_STICKY
        } else {
            START_NOT_STICKY
        }
        return super.onStartCommand(intent, flags, startId)
    }
}