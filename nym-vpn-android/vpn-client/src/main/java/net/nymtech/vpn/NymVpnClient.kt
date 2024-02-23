package net.nymtech.vpn

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

import timber.log.Timber

class NymVpnClient : VpnClient {

    private val _statistics = MutableStateFlow(VpnStatistics())
    override val statistics: StateFlow<VpnStatistics> = _statistics.asStateFlow()
    val scope = CoroutineScope(Dispatchers.IO)


    private var job : Job? = null
    override fun prepare(context: Context) : Intent? {
        return VpnService.prepare(context)
    }

    override fun connect() {
        job = scope.launch {
            var seconds = 0L
            do {
                _statistics.value = _statistics.value.copy(
                    connectionSeconds = seconds
                )
                delay(1000)
                seconds++
            } while (true)
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