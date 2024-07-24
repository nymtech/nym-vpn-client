package net.nymtech.nymvpn.ui.screens.settings.credential

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.withContext
import net.nymtech.nymvpn.data.SettingsRepository
import net.nymtech.nymvpn.util.Constants
import net.nymtech.vpnclient.VpnClient
import timber.log.Timber
import java.time.Instant
import javax.inject.Inject
import javax.inject.Provider

@HiltViewModel
class CredentialViewModel
@Inject
constructor(
	private val vpnClient: Provider<VpnClient>,
	private val settingsRepository: SettingsRepository,
) : ViewModel() {

	suspend fun onImportCredential(credential: String): Result<Instant?> {
		val trimmedCred = credential.trim()
		return withContext(viewModelScope.coroutineContext) {
			vpnClient.get().importCredential(trimmedCred, Constants.NATIVE_STORAGE_PATH).onSuccess {
				Timber.d("Imported credential successfully")
				it?.let {
					settingsRepository.saveCredentialExpiry(it)
				}
			}.onFailure { Timber.e(it) }
		}
	}
}
