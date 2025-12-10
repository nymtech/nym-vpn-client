package net.nymtech.nymvpn.ui.screens.settings.dns

import androidx.lifecycle.ViewModel
import dagger.hilt.android.lifecycle.HiltViewModel
import net.nymtech.nymvpn.data.SettingsRepository
import net.nymtech.nymvpn.manager.backend.BackendManager
import net.nymtech.nymvpn.manager.environment.EnvironmentManager
import javax.inject.Inject

@HiltViewModel
class DnsViewModel @Inject constructor(
	private val backendManager: BackendManager,
	private val settingsRepository: SettingsRepository,
	private val environmentManager: EnvironmentManager,
) : ViewModel() {
}
