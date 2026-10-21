package net.nymtech.nymvpn.ui.screens.account.delete

import androidx.lifecycle.ViewModel
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import javax.inject.Inject

@HiltViewModel
class DeleteViewModel @Inject constructor() : ViewModel() {

	private val _uiState = MutableStateFlow(DeleteUiState())
	val uiState: StateFlow<DeleteUiState> = _uiState.asStateFlow()
}
