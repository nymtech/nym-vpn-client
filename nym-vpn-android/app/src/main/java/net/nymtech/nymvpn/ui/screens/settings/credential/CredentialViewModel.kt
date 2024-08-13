package net.nymtech.nymvpn.ui.screens.settings.credential

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.withContext
import net.nymtech.nymvpn.data.SettingsRepository
import net.nymtech.nymvpn.service.tunnel.TunnelManager
import timber.log.Timber
import java.time.Instant
import javax.inject.Inject

@HiltViewModel
class CredentialViewModel
@Inject
constructor(
	private val settingsRepository: SettingsRepository,
	private val tunnelManager: TunnelManager,
) : ViewModel() {

	suspend fun onImportCredential(credential: String): Result<Instant?> {
		val trimmedCred = credential.trim()
		return withContext(viewModelScope.coroutineContext) {
			tunnelManager.importCredential(trimmedCred).onSuccess {
				Timber.d("Imported credential successfully")
				it?.let {
					settingsRepository.saveCredentialExpiry(it)
				}
			}.onFailure { Timber.e(it) }
		}
	}
}
