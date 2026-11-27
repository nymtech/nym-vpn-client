package net.nymtech.nymvpn.ui.screens.settings.diagnostic

import android.content.Context
import androidx.core.content.FileProvider
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.Job
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

	// Starts loading: the run begins as soon as the screen opens, so the first frame is the spinner.
	private val _uiState = MutableStateFlow(DiagnosticUiState(isLoading = true))
	val uiState: StateFlow<DiagnosticUiState> = _uiState.asStateFlow()

	private var runJob: Job? = null

	init {
		runDiagnostics()
	}

	fun runDiagnostics() {
		// Ignore a tap while a run is active; loading is set here, not in the coroutine, so the button hides at once.
		if (runJob?.isActive == true) return
		_uiState.update { it.copy(isLoading = true, report = null) }
		runJob = viewModelScope.launch(ioDispatcher) {
			runCatching {
				backendManager.runDiagnostic()
			}.onSuccess { report ->
				// Most failures arrive here as no report rather than an exception (core not ready, or a swallowed error).
				if (report == null) Timber.w("DiagnosticsRunNoReport")
				_uiState.update { it.copy(isLoading = false, report = report) }
			}.onFailure { t ->
				Timber.e(t, "DiagnosticsRunFailed")
				_uiState.update { it.copy(isLoading = false) }
			}
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
