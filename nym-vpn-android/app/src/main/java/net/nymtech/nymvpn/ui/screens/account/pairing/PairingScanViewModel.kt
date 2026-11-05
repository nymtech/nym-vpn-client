package net.nymtech.nymvpn.ui.screens.account.pairing

import androidx.lifecycle.ViewModel
import dagger.hilt.android.lifecycle.HiltViewModel
import net.nymtech.nymvpn.manager.pairing.PairingQrValidator
import javax.inject.Inject

@HiltViewModel
class PairingScanViewModel @Inject constructor(val validator: PairingQrValidator) : ViewModel()
