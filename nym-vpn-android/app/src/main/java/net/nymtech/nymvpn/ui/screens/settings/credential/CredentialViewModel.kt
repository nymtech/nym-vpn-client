package net.nymtech.nymvpn.ui.screens.settings.credential

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import net.nymtech.nymvpn.data.SecretsRepository
import net.nymtech.vpn.NymVpnClient
import javax.inject.Inject
import javax.inject.Provider

@HiltViewModel
class CredentialViewModel
@Inject
constructor(
	private val secretsRepository: Provider<SecretsRepository>,
) : ViewModel() {
	fun onImportCredential(credential: String): Result<Unit> {
		val trimmedCred = credential.trim()
		return NymVpnClient.validateCredential(trimmedCred).onSuccess {
			saveCredential(credential)
		}
	}

	private fun saveCredential(credential: String) = viewModelScope.launch(Dispatchers.IO) {
		secretsRepository.get().saveCredential(credential)
	}
}
