package net.nymtech.vpn

import android.content.Context
import android.content.Intent
import android.net.VpnService
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.Job
import kotlinx.coroutines.cancel
import kotlinx.coroutines.channels.awaitClose
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.callbackFlow
import kotlinx.coroutines.launch
import kotlinx.coroutines.runBlocking
import kotlinx.coroutines.withContext
import net.nymtech.logcat_helper.LogcatHelper
import net.nymtech.logcat_helper.model.LogLevel
import net.nymtech.vpn.model.ClientState
import net.nymtech.vpn.model.Country
import net.nymtech.vpn.model.ErrorState
import net.nymtech.vpn.model.VpnMode
import net.nymtech.vpn.model.VpnState
import net.nymtech.vpn.util.ServiceManager
import net.nymtech.vpn.util.safeCollect
import net.nymtech.vpn_client.BuildConfig
import nym_vpn_lib.EntryPoint
import nym_vpn_lib.ExitPoint
import nym_vpn_lib.FfiException
import nym_vpn_lib.VpnConfig
import nym_vpn_lib.getGatewayCountries
import nym_vpn_lib.getLowLatencyEntryCountry
import nym_vpn_lib.runVpn
import timber.log.Timber
import java.net.URL

//TODO change to builder pattern?
object NymVpnClient : VpnClient {

    private val apiUrl = URL(BuildConfig.API_URL)
    private val explorerUrl = URL(BuildConfig.EXPLORER_URL)
    private val scope = CoroutineScope(Dispatchers.IO)

    private val _state = MutableStateFlow(ClientState())
    override val stateFlow: Flow<ClientState> = _state.asStateFlow()
    override fun getState(): ClientState {
        return _state.value
    }

    override suspend fun gateways(exitOnly: Boolean) : Set<Country> {
        return withContext(CoroutineScope(Dispatchers.IO).coroutineContext) {
            getGatewayCountries(apiUrl,explorerUrl,exitOnly).map {
                Country(isoCode = it.twoLetterIsoCountryCode)
            }.toSet()
        }
    }

    override suspend fun getLowLatencyEntryCountryCode(): Country {
        return withContext(CoroutineScope(Dispatchers.IO).coroutineContext) {
            Country(isoCode = getLowLatencyEntryCountry(apiUrl, explorerUrl).twoLetterIsoCountryCode, isLowLatency = true)
        }
    }

    private var statusJob: Job? = null
    override fun configure(entryPoint: EntryPoint, exitPoint: ExitPoint, mode: VpnMode) {
        _state.value = _state.value.copy(
            entryPoint = entryPoint,
            exitPoint = exitPoint,
            mode = mode
        )
    }

    override fun prepare(context : Context): Intent? {
        return VpnService.prepare(context)
    }

    override fun start(context: Context) {
        clearErrorStatus()
        statusJob = collectLogStatus(context)
        ServiceManager.startVpnService(context)
    }

    override fun startForeground(context: Context) {
        clearErrorStatus()
        statusJob = collectLogStatus(context)
        ServiceManager.startVpnServiceForeground(context)
    }
    internal fun connect() {
        //TODO refactor
        if(_state.value.exitPoint != null && _state.value.entryPoint != null) {
            try {
                runVpn(VpnConfig(
                    apiUrl, explorerUrl,
                    _state.value.entryPoint!!, _state.value.exitPoint!!, isTwoHop(_state.value.mode)))
            } catch (e : Exception) {
                Timber.e(e)
            }
        }
    }

    private fun isTwoHop(mode : VpnMode) : Boolean = when(mode) {
        VpnMode.TWO_HOP_MIXNET -> true
        else -> false
    }

    private fun collectLogStatus(context: Context) = scope.launch {
        launch {
            callbackFlow {
                LogcatHelper.logs {
                    trySend(it)
                }
                awaitClose { cancel() }
            }.safeCollect {
                when(it.level) {
                    LogLevel.ERROR -> {
                        //TODO probably don't want to handle all errors this way
                        cancel()
                        setErrorState(it.message)
                        disconnect(context)
                        statusJob?.cancel()
                    }
                    LogLevel.INFO -> {
                        parseLibInfo(it.message)
                    }
                    else -> Unit
                }
            }
        }
        launch {
            var seconds = 0L
            do {
                if(_state.value.vpnState == VpnState.Up) {
                    _state.value = _state.value.copy(
                        statistics = _state.value.statistics.copy(
                            connectionSeconds = seconds
                        )
                    )
                    seconds++
                }
                delay(1000)
            } while (true)
        }
    }

    private fun parseLibInfo(message : String) {
        //TODO make this more robust in the future
        with(message){
            when {
                contains("Mixnet processor is running") -> setVpnState(VpnState.Up)
                contains("Nym VPN has shut down") -> setVpnState(VpnState.Down)
                contains("Connecting to IP packet router") -> setVpnState(VpnState.Connecting.EstablishingConnection)
            }
        }
    }

    private fun clearErrorStatus() {
        _state.value = _state.value.copy(
            errorState = ErrorState.None
        )
    }
    private fun setErrorState(message : String) {
        _state.value = _state.value.copy(
            errorState = ErrorState.LibraryError(message)
        )
    }

    internal fun setVpnState(state : VpnState) {
        _state.value = _state.value.copy(
            vpnState = state
        )
    }


    override fun disconnect(context: Context) {
        statusJob?.cancel()
        _state.value = _state.value.copy(
            statistics = _state.value.statistics.copy(
                connectionSeconds = null
            ),
        )
        ServiceManager.stopVpnService(context)
    }
    const val ENTRY_POINT_EXTRA_KEY = "entryPoint"
    const val EXIT_POINT_EXTRA_KEY = "exitPoint"
    const val TWO_HOP_EXTRA_KEY = "twoHop"
}