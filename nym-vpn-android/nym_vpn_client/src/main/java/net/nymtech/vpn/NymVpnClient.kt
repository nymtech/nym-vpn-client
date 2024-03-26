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
import kotlinx.coroutines.withContext
import net.nymtech.logcat_helper.LogcatHelper
import net.nymtech.logcat_helper.model.LogLevel
import net.nymtech.vpn.model.ClientState
import net.nymtech.vpn.model.EntryPoint
import net.nymtech.vpn.model.ErrorState
import net.nymtech.vpn.model.ExitPoint
import net.nymtech.vpn.model.VpnMode
import net.nymtech.vpn.model.VpnState
import net.nymtech.vpn.util.Constants
import net.nymtech.vpn.util.ServiceManager
import net.nymtech.vpn.util.safeCollect
import net.nymtech.vpn_client.BuildConfig
import timber.log.Timber
import uniffi.nym_vpn_lib.Country
import uniffi.nym_vpn_lib.getGatewayCountries
import uniffi.nym_vpn_lib.getLowLatencyEntryCountry

object NymVpnClient : VpnClient {

    init {
        Constants.setupEnvironment()
        System.loadLibrary(Constants.NYM_VPN_LIB)
        Timber.i( "Loaded native library in client")
    }

    private val _state = MutableStateFlow(ClientState())
    override val stateFlow: Flow<ClientState> = _state.asStateFlow()
    override fun getState(): ClientState {
        return _state.value
    }

    override suspend fun gateways(exitOnly: Boolean) : List<String> {
        return withContext(CoroutineScope(Dispatchers.IO).coroutineContext) {
            val gateways = getGatewayCountries(BuildConfig.API_URL,BuildConfig.EXPLORER_URL,exitOnly)
            gateways.map {
                (it as Country.Code).value
            }
        }
    }

    override suspend fun getLowLatencyEntryCountryCode(): String {
        return withContext(CoroutineScope(Dispatchers.IO).coroutineContext) {
            getLowLatencyEntryCountry(BuildConfig.API_URL, BuildConfig.EXPLORER_URL).let {
                (it as Country.Code).value
            }
        }
    }


    private val scope = CoroutineScope(Dispatchers.IO)

    private var statusJob: Job? = null
    override fun prepare(context : Context): Intent? {
        return VpnService.prepare(context)
    }

    override fun connect(context: Context, entryPoint: EntryPoint, exitPoint: ExitPoint, mode: VpnMode) {
        clearErrorStatus()
        setMode(mode)
        val extras = mapOf(
            ENTRY_POINT_EXTRA_KEY to entryPoint.toLibString(),
            EXIT_POINT_EXTRA_KEY to exitPoint.toLibString(),
            TWO_HOP_EXTRA_KEY to isTwoHop(mode).toString()
        )
        //TODO fix logic for more modes later
        statusJob = collectLogStatus(context)
        ServiceManager.startVpnService(context, extras)
    }

    override fun connectForeground(
        context: Context,
        entryPoint: EntryPoint,
        exitPoint: ExitPoint,
        mode: VpnMode
    ) {
        clearErrorStatus()
        setMode(mode)
        val extras = mapOf(
            ENTRY_POINT_EXTRA_KEY to entryPoint.toLibString(),
            EXIT_POINT_EXTRA_KEY to exitPoint.toLibString(),
            TWO_HOP_EXTRA_KEY to isTwoHop(mode).toString()
        )
        statusJob = collectLogStatus(context)
        ServiceManager.startVpnServiceForeground(context, extras)
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

    private fun setMode(mode : VpnMode) {
        _state.value = _state.value.copy(
            mode = mode
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