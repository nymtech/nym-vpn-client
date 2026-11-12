package net.nymtech.nymvpn.ui.screens.scan

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.receiveAsFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import net.nymtech.nymvpn.R
import net.nymtech.qrscanner.model.QrScanValidation
import timber.log.Timber
import javax.inject.Inject

@HiltViewModel
class RedeemCodeViewModel @Inject constructor(private val validator: ScannedCodeValidator) : ViewModel() {

	private val _uiState = MutableStateFlow(RedeemCodeUiState())
	val uiState: StateFlow<RedeemCodeUiState> = _uiState.asStateFlow()

	// A channel keeps the result until collected, e.g. across activity recreation.
	private val _accepted = Channel<ScannedQrCode>(Channel.CONFLATED)
	val accepted: Flow<ScannedQrCode> = _accepted.receiveAsFlow()

	fun onCodeChange(code: String) {
		_uiState.update { it.copy(code = code, errorRes = null) }
	}

	// The view model outlives the sheet, so closing it clears the code.
	fun reset() {
		if (!_uiState.value.loading) _uiState.value = RedeemCodeUiState()
	}

	fun submit() {
		val contents = _uiState.value.code
		if (_uiState.value.loading || contents.isBlank()) return
		val code = ScannedQrCode.parse(contents)
		if (code == null) {
			_uiState.update { it.copy(errorRes = R.string.redeem_code_invalid) }
			return
		}
		_uiState.update { it.copy(loading = true, errorRes = null) }
		viewModelScope.launch {
			when (val validation = validate(contents)) {
				QrScanValidation.Valid -> {
					_uiState.update { it.copy(loading = false) }
					_accepted.send(code)
				}
				is QrScanValidation.Invalid -> _uiState.update { it.copy(loading = false, errorRes = validation.messageRes ?: R.string.redeem_code_invalid) }
			}
		}
	}

	private suspend fun validate(contents: String): QrScanValidation = try {
		validator.validate(contents)
	} catch (e: CancellationException) {
		throw e
	} catch (e: Exception) {
		Timber.tag(TAG).w("Code validation failed: %s", e::class.java.simpleName)
		QrScanValidation.Invalid()
	}

	private companion object {
		const val TAG = "ui-redeem-code-vm"
	}
}
