package net.nymtech.vpn.backend.api

import android.app.Service
import android.content.Intent
import android.os.Build
import android.os.IBinder
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.withContext
import net.nymtech.vpn.backend.ConnectInitRequest
import net.nymtech.vpn.backend.ConnectRequest
import net.nymtech.vpn.backend.ConnectResult
import net.nymtech.vpn.backend.Tunnel
import net.nymtech.vpn.backend.service.VpnService
import net.nymtech.vpn.model.NymGateway
import nym_vpn_lib_types.AccountControllerState
import nym_vpn_lib_types.FeatureFlags
import nym_vpn_lib_types.GatewayType
import nym_vpn_lib_types.ListGatewaysOptions
import nym_vpn_lib_types.NetworkCompatibility
import nym_vpn_lib_types.ParsedAccountLinks
import nym_vpn_lib_types.StoreAccountRequest
import nym_vpn_lib_types.SystemMessage
import timber.log.Timber

class VpnApiService : Service() {

	companion object {
		private const val TAG = "core-vpn-api"
	}

	class LocalBinder(private val api: VpnServiceApi) : android.os.Binder() {
		fun api(): VpnServiceApi = api
	}

	private val apiImpl: VpnServiceApi = object : VpnServiceApi {

		override suspend fun init(request: ConnectInitRequest): ConnectResult {
			val s = awaitOrStartVpnService()
			return s.initFromApi(request)
		}

		override fun getState(): Tunnel.State {
			val s = VpnService.serviceFlow.value
			return s?.getState() ?: Tunnel.State.Down
		}

		override val events = VpnService.serviceFlow
			.filterNotNull()
			.flatMapLatest { it.events }

		override suspend fun connect(request: ConnectRequest): ConnectResult {
			val s = awaitOrStartVpnService()
			return s.connectFromApi(request)
		}

		override suspend fun disconnect(): ConnectResult {
			val s = VpnService.serviceFlow.value ?: return ConnectResult.Ok
			return s.disconnectFromApi()
		}

		override suspend fun isMnemonicStored(): Boolean {
			val s = awaitOrStartVpnService()
			return s.tryWithCoreSender { it.isAccountStored() } ?: false
		}

		override suspend fun storeMnemonic(mnemonic: String) {
			val s = awaitOrStartVpnService()
			s.requireCoreSender { it.storeAccount(StoreAccountRequest.Vpn(mnemonic)) }
		}

		override suspend fun removeMnemonic() {
			val s = awaitOrStartVpnService()
			s.requireCoreSender { it.forgetAccount() }
		}

		override suspend fun getAccountState(): AccountControllerState {
			val s = awaitOrStartVpnService()
			return s.requireCoreSender { it.getAccountState() }
		}

		override suspend fun getAccountLinks(locale: String): ParsedAccountLinks? {
			val s = awaitOrStartVpnService()
			return s.tryWithCoreSender { it.getAccountLinks(locale) }
		}

		override suspend fun getSystemMessages(): List<SystemMessage> {
			val s = awaitOrStartVpnService()
			return s.tryWithCoreSender { it.getSystemMessages() } ?: emptyList()
		}

		override suspend fun getGateways(type: GatewayType): List<NymGateway> {
			val s = awaitOrStartVpnService()
			return s.tryWithCoreSender {
				it.listGateways(ListGatewaysOptions(gwType = type, userAgent = null))
					.map(NymGateway::from)
			} ?: emptyList()
		}

		override suspend fun getNetworkVersions(): NetworkCompatibility? {
			val s = awaitOrStartVpnService()
			return s.tryWithCoreSender { it.getNetworkCompatibility() }
		}

		override suspend fun getDeviceIdentity(): String? {
			val s = awaitOrStartVpnService()
			return s.tryWithCoreSender { it.getDeviceIdentity() }
		}

		override suspend fun getAccountIdentity(): String? {
			val s = awaitOrStartVpnService()
			return s.tryWithCoreSender { it.getAccountIdentity() }
		}

		override suspend fun getFeatureFlags(): FeatureFlags? {
			val s = awaitOrStartVpnService()
			return s.tryWithCoreSender { it.getFeatureFlags() }
		}
	}

	private val binder = LocalBinder(apiImpl)

	override fun onBind(intent: Intent?): IBinder {
		Timber.tag(TAG).i("onBind action=%s", intent?.action)
		return binder
	}

	private suspend fun awaitVpnService(): VpnService = VpnService.serviceFlow.filterNotNull().first()

	private suspend fun awaitOrStartVpnService(): VpnService {
		VpnService.serviceFlow.value?.let { return it }

		withContext(Dispatchers.Main.immediate) {
			runCatching {
				val intent = Intent(this@VpnApiService, VpnService::class.java).apply {
					action = VpnService.ACTION_START_FROM_API
				}
				if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) startForegroundService(intent) else startService(intent)
			}.onFailure {
				Timber.tag(TAG).e(it, "startService(VpnService) failed")
			}
		}

		return awaitVpnService()
	}
}
