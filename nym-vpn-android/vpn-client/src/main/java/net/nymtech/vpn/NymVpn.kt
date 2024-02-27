package net.nymtech.vpn

import android.content.Context
import android.content.Intent
import android.net.VpnService
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.Job
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch
import net.nymtech.vpn.model.EntryPoint
import net.nymtech.vpn.model.ExitPoint
import net.nymtech.vpn.model.VpnState
import net.nymtech.vpn.model.VpnStatistics
import net.nymtech.vpn.util.ServiceManager

object NymVpn : VpnClient {

    private val _state = MutableStateFlow(VpnState.DOWN)
    override val stateFlow: Flow<VpnState> = _state.asStateFlow()

    private val _statistics = MutableStateFlow(VpnStatistics())
    override val statistics: StateFlow<VpnStatistics> = _statistics.asStateFlow()

    private val scope = CoroutineScope(Dispatchers.IO)

    private var statsJob: Job? = null
    override fun prepare(context : Context): Intent? {
        return VpnService.prepare(context)
    }

    override fun connect(context: Context, entryPoint: EntryPoint, exitPoint: ExitPoint, isTwoHop: Boolean) {
        val extras = mapOf(
            ENTRY_POINT_EXTRA_KEY to entryPoint.toString(),
            EXIT_POINT_EXTRA_KEY to exitPoint.toString(),
            TWO_HOP_EXTRA_KEY to isTwoHop.toString()
        )
        statsJob = gatherStatistics()
        ServiceManager.startVpnService(context, extras)
    }

    private fun gatherStatistics() = scope.launch {
        var seconds = 0L
        do {
            _statistics.value = _statistics.value.copy(
                connectionSeconds = seconds
            )
            delay(1000)
            seconds++
        } while (true)
    }

    internal fun setState(state : VpnState) = scope.launch {
        _state.value = state
    }


    override fun disconnect(context: Context) {
        statsJob?.cancel()
        _statistics.value = _statistics.value.copy(
            connectionSeconds = null
        )
        ServiceManager.stopVpnService(context)
    }
    const val ENTRY_POINT_EXTRA_KEY = "entryPoint"
    const val EXIT_POINT_EXTRA_KEY = "exitPoint"
    const val TWO_HOP_EXTRA_KEY = "twoHop"
}