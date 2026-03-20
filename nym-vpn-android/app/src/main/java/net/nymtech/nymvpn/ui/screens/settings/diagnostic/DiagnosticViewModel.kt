package net.nymtech.nymvpn.ui.screens.settings.diagnostic

import android.content.Context
import androidx.core.content.FileProvider
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.di.qualifiers.IoDispatcher
import net.nymtech.nymvpn.manager.backend.BackendManager
import net.nymtech.nymvpn.util.extensions.launchShareFile
import timber.log.Timber
import java.io.File
import javax.inject.Inject

@HiltViewModel
class DiagnosticViewModel @Inject constructor(private val backendManager: BackendManager, @IoDispatcher private val ioDispatcher: CoroutineDispatcher) : ViewModel() {

	private val _uiState = MutableStateFlow(DiagnosticUiState())
	val uiState: StateFlow<DiagnosticUiState> = _uiState.asStateFlow()

	fun runDiagnostics() = viewModelScope.launch(ioDispatcher) {
		_uiState.update { it.copy(isLoading = true, error = null) }
		runCatching {
			backendManager.runDiagnostic()
		}.onSuccess { report ->
			_uiState.update { it.copy(isLoading = false, report = report) }
		}.onFailure { t ->
			Timber.e(t, "DiagnosticsRunFailed")
			_uiState.update { it.copy(isLoading = false, error = t.message) }
		}
	}

	fun shareReport(context: Context) = viewModelScope.launch(ioDispatcher) {
		val report = _uiState.value.report ?: return@launch
		runCatching {
			val sharePath = File(context.filesDir, "external_files")
			if (!sharePath.exists()) sharePath.mkdir()
			val file = File(sharePath, "diagnostic_report.json")
			file.writeText(report)
			val uri = FileProvider.getUriForFile(context, context.getString(R.string.provider), file)
			context.launchShareFile(uri)
		}.onFailure { t ->
			Timber.e(t, "DiagnosticsShareFailed")
		}
	}
}
