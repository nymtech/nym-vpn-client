package net.nymtech.nymvpn.ui.screens.settings.credential

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import net.nymtech.nymvpn.data.SecretsRepository
import net.nymtech.vpn.VpnClient
import java.time.Instant
import javax.inject.Inject
import javax.inject.Provider

@HiltViewModel
class CredentialViewModel
@Inject
constructor(
	private val secretsRepository: Provider<SecretsRepository>,
	private val vpnClient: Provider<VpnClient>,
) : ViewModel() {

	suspend fun onImportCredential(credential: String): Result<Instant> {
		val trimmedCred = credential.trim()
		return withContext(viewModelScope.coroutineContext) {
			vpnClient.get().validateCredential(trimmedCred).onSuccess {
				saveCredential(credential)
			}
		}
	}

	private fun saveCredential(credential: String) = viewModelScope.launch {
		secretsRepository.get().saveCredential(credential)
	}
}
