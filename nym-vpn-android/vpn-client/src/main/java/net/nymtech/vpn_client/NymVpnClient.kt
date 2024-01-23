package net.nymtech.vpn_client

import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.Job
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch

class NymVpnClient : VpnClient {

    private val _statistics = MutableStateFlow(VpnStatistics())
    override val statistics: StateFlow<VpnStatistics> = _statistics.asStateFlow()

    private val scope = CoroutineScope(Dispatchers.IO)

    private var job : Job? = null

    //TODO maybe change this to entrypoint exitpoint objects
    override fun connect(entryIso: String, exitIso: String) {
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