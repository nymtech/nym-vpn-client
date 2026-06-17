package net.nymtech.nymvpn.ui.screens.settings.notifications

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.launch
import net.nymtech.nymvpn.data.config.VpnConfigRepository
import net.nymtech.vpn.config.CoreVpnConfigUpdate
import timber.log.Timber
import javax.inject.Inject

@HiltViewModel
class NotificationsViewModel @Inject constructor(private val vpnConfigRepository: VpnConfigRepository) : ViewModel() {

	companion object {
		private const val TAG = "notifications-vm"
	}

	fun onNodeFamiliesEnabled(enabled: Boolean) = viewModelScope.launch {
		runCatching {
			vpnConfigRepository.apply(CoreVpnConfigUpdate.SetNodeFamiliesNotificationsEnabled(enabled))
		}.onFailure { Timber.tag(TAG).e(it, "Failed to update node families notifications setting") }
	}
}
