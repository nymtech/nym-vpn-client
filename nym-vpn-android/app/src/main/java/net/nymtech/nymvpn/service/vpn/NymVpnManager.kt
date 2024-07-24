package net.nymtech.nymvpn.service.vpn

import android.content.Context
import net.nymtech.nymvpn.NymVpn
import net.nymtech.nymvpn.data.SettingsRepository
import net.nymtech.vpnclient.VpnClient
import net.nymtech.vpnclient.util.InvalidCredentialException
import java.time.Instant
import javax.inject.Inject
import javax.inject.Provider

class NymVpnManager @Inject constructor(
	private val settingsRepository: SettingsRepository,
	private val vpnClient: Provider<VpnClient>,
	private val context: Context,
) : VpnManager {
	override suspend fun stopVpn(foreground: Boolean) {
		vpnClient.get().stop(foreground)
		NymVpn.requestTileServiceStateUpdate()
	}

	override suspend fun startVpn(foreground: Boolean): Result<Unit> {
		val entryCountry = settingsRepository.getFirstHopCountry()
		val exitCountry = settingsRepository.getLastHopCountry()
		val credentialExpiry = settingsRepository.getCredentialExpiry()
		val mode = settingsRepository.getVpnMode()
		return if (credentialExpiry != null && credentialExpiry.isAfter(Instant.now())) {
			val entry = entryCountry.toEntryPoint()
			val exit = exitCountry.toExitPoint()
			return vpnClient.get().apply {
				this.mode = mode
				this.exitPoint = exit
				this.entryPoint = entry
			}.start(context, true).also {
				NymVpn.requestTileServiceStateUpdate()
			}
		} else {
			Result.failure(InvalidCredentialException("No credential found"))
		}
	}
}
