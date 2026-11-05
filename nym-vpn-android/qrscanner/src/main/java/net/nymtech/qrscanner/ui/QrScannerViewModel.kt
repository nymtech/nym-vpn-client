package net.nymtech.qrscanner.ui

import androidx.annotation.StringRes
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.Job
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import net.nymtech.qrscanner.QrScanValidator
import net.nymtech.qrscanner.model.QrScanMode
import net.nymtech.qrscanner.model.QrScanResult
import net.nymtech.qrscanner.model.QrScanValidation
import timber.log.Timber
import kotlin.time.Duration
import kotlin.time.Duration.Companion.seconds
import kotlin.time.TimeMark
import kotlin.time.TimeSource

internal class QrScannerViewModel(
	private val mode: QrScanMode,
	private val validator: QrScanValidator,
	private val invalidCodeMessageDuration: Duration = 2.seconds,
	private val timeSource: TimeSource = TimeSource.Monotonic,
) : ViewModel() {
	private val _uiState = MutableStateFlow(QrScannerUiState(invalidCodeRes = mode.invalidCodeRes))
	val uiState: StateFlow<QrScannerUiState> = _uiState.asStateFlow()

	private var validating = false

	// Stops a rejected code still in view from being re-validated on every frame.
	private var lastRejection: Rejection? = null
	private var hideInvalidCodeJob: Job? = null

	private class Rejection(val contents: String, val retryAt: TimeMark?)

	fun onCodeScan(contents: String) {
		if (_uiState.value.result != null || validating) return
		val rejection = lastRejection
		if (rejection != null && rejection.contents == contents && rejection.retryAt?.hasPassedNow() != true) {
			showInvalidCode(_uiState.value.invalidCodeRes)
			return
		}
		validating = true
		viewModelScope.launch {
			val validation = validate(contents)
			validating = false
			when (validation) {
				QrScanValidation.Valid -> {
					hideInvalidCodeJob?.cancel()
					lastRejection = null
					_uiState.update { it.copy(result = mode.toResult(contents), showInvalidCode = false) }
				}
				is QrScanValidation.Invalid -> {
					lastRejection = Rejection(contents, validation.retryAfter?.let { timeSource.markNow() + it })
					showInvalidCode(validation.messageRes ?: mode.invalidCodeRes)
				}
			}
		}
	}

	fun onFallback() {
		if (_uiState.value.result != null) return
		_uiState.update { it.copy(result = QrScanResult.Fallback) }
	}

	private suspend fun validate(contents: String): QrScanValidation = try {
		validator.validate(contents)
	} catch (e: CancellationException) {
		throw e
	} catch (e: Exception) {
		Timber.w(e, "QR validator failed for mode %s", mode)
		QrScanValidation.Invalid()
	}

	private fun showInvalidCode(@StringRes messageRes: Int) {
		_uiState.update { it.copy(showInvalidCode = true, invalidCodeRes = messageRes) }
		hideInvalidCodeJob?.cancel()
		hideInvalidCodeJob = viewModelScope.launch {
			delay(invalidCodeMessageDuration)
			_uiState.update { it.copy(showInvalidCode = false) }
		}
	}
}
