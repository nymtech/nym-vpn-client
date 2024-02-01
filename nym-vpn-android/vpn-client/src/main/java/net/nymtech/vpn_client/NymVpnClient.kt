package net.nymtech.vpn_client

import net.nymtech.uniffi.lib.NymVPN
import android.content.Context
import android.content.Intent
import android.net.VpnService
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.DelicateCoroutinesApi
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.GlobalScope
import kotlinx.coroutines.Job
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch
import net.nymtech.NymVpnService

import timber.log.Timber

class NymVpnClient : VpnClient {

    private val _statistics = MutableStateFlow(VpnStatistics())
    override val statistics: StateFlow<VpnStatistics> = _statistics.asStateFlow()

    private val scope = CoroutineScope(Dispatchers.IO)

    private val nymVPN : NymVPN = NymVPN()

    private var job : Job? = null
    override fun prepare(context: Context) : Intent? {
        return VpnService.prepare(context)
    }

    @OptIn(DelicateCoroutinesApi::class)
    override fun connect(entryIso: String, exitIso: String, vpnService: NymVpnService, interfaceFd : Int) {
        Timber.d("Starting job")
        val entry = "{ \"Location\": { \"location\": \"FR\" }}"
        val exit = "{ \"Location\": { \"location\": \"FR\" }}"
        GlobalScope.launch(Dispatchers.IO) {
            nymVPN.init("https://sandbox-nym-api1.nymtech.net/api",entry,exit,vpnService,interfaceFd)
            delay(1000)
            nymVPN.run()
        }


//        job = scope.launch(Dispatchers.IO) {
//            var seconds = 0L
//            do {
//                _statistics.value = _statistics.value.copy(
//                    connectionSeconds = seconds
//                )
//                delay(1000)
//                seconds++
//            } while (true)
//        }
    }

    override fun disconnect() {
        //TODO reset statistics here too
        _statistics.value = _statistics.value.copy(
            connectionSeconds = null
        )
        job?.cancel()
    }
}