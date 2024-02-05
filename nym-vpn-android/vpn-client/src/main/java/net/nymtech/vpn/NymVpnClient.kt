package net.nymtech.vpn

import net.nymtech.uniffi.lib.NymVPN
import android.content.Context
import android.content.Intent
import android.net.VpnService
import kotlinx.coroutines.DelicateCoroutinesApi
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.GlobalScope
import kotlinx.coroutines.Job
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch

import timber.log.Timber

class NymVpnClient : VpnClient {

    private val _statistics = MutableStateFlow(VpnStatistics())
    override val statistics: StateFlow<VpnStatistics> = _statistics.asStateFlow()

    private val nymVPN : NymVPN = NymVPN()

    private var job : Job? = null
    override fun prepare(context: Context) : Intent? {
        return VpnService.prepare(context)
    }

    @OptIn(DelicateCoroutinesApi::class)
    override fun connect(entryIso: String, exitIso: String, vpnService: NymVpnService) {
        Timber.d("Starting job")
        val entry = "{ \"Location\": { \"location\": \"FR\" }}"
        val exit = "{ \"Location\": { \"location\": \"FR\" }}"
        GlobalScope.launch(Dispatchers.IO) {
            nymVPN.init("https://sandbox-nym-api1.nymtech.net/api",entry,exit,vpnService)
            delay(1000)
            nymVPN.run()
        }
    }

    override fun disconnect() {
        //TODO reset statistics here too
        _statistics.value = _statistics.value.copy(
            connectionSeconds = null
        )
        job?.cancel()
    }
}