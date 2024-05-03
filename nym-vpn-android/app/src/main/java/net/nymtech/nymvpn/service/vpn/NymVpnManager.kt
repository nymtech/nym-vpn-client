package net.nymtech.nymvpn.service.vpn

import android.content.Context
import net.nymtech.nymvpn.NymVpn
import net.nymtech.nymvpn.data.SecretsRepository
import net.nymtech.nymvpn.data.SettingsRepository
import net.nymtech.vpn.VpnClient
import net.nymtech.vpn.util.InvalidCredentialException
import javax.inject.Inject
import javax.inject.Provider

class NymVpnManager @Inject constructor(
	private val settingsRepository: SettingsRepository,
	private val secretsRepository: Provider<SecretsRepository>,
	private val vpnClient: Provider<VpnClient>,
) : VpnManager {
	override fun stopVpn(context: Context, foreground: Boolean) {
		vpnClient.get().stop(NymVpn.instance, foreground)
		NymVpn.requestTileServiceStateUpdate()
	}

	override suspend fun startVpn(context: Context, foreground: Boolean): Result<Unit> {
		val entryCountry = settingsRepository.getFirstHopCountry()
		val exitCountry = settingsRepository.getLastHopCountry()
		val credential = secretsRepository.get().getCredential()
		val mode = settingsRepository.getVpnMode()
		return if (credential != null) {
			val entry = entryCountry.toEntryPoint()
			val exit = exitCountry.toExitPoint()
			try {
				vpnClient.get().apply {
					this.mode = mode
					this.exitPoint = exit
					this.entryPoint = entry
				}.start(context, credential, true)
				NymVpn.requestTileServiceStateUpdate()
				Result.success(Unit)
			} catch (e: InvalidCredentialException) {
				Result.failure(e)
			}
		} else {
			Result.failure(InvalidCredentialException("No credential found"))
		}
	}
}
