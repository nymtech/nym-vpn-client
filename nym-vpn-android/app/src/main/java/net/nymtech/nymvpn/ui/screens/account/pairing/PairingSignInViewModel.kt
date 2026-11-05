package net.nymtech.nymvpn.ui.screens.account.pairing

import androidx.lifecycle.ViewModel
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.launch
import net.nymtech.nymvpn.di.qualifiers.ApplicationScope
import net.nymtech.nymvpn.manager.pairing.PairingManager
import net.nymtech.nymvpn.manager.pairing.model.PairingState
import javax.inject.Inject

@HiltViewModel
class PairingSignInViewModel @Inject constructor(private val pairingManager: PairingManager, @ApplicationScope private val appScope: CoroutineScope) : ViewModel() {

	val state: StateFlow<PairingState> = pairingManager.state

	override fun onCleared() {
		appScope.launch { pairingManager.cancel() }
	}
}
