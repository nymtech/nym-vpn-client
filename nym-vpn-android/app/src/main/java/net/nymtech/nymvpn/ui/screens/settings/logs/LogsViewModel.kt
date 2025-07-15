package net.nymtech.nymvpn.ui.screens.settings.logs

import android.content.Context
import androidx.compose.runtime.mutableStateListOf
import androidx.core.content.FileProvider
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.Job
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.collectLatest
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import net.nymtech.logcatutil.LogReader
import net.nymtech.logcatutil.model.LogMessage
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.di.qualifiers.IoDispatcher
import net.nymtech.nymvpn.di.qualifiers.MainDispatcher
import net.nymtech.nymvpn.util.Constants
import net.nymtech.nymvpn.util.extensions.chunked
import net.nymtech.nymvpn.util.extensions.launchShareFile
import timber.log.Timber
import java.io.File
import java.time.Duration
import java.time.Instant
import javax.inject.Inject

@HiltViewModel
class LogsViewModel @Inject constructor(
	private val logReader: LogReader,
	@IoDispatcher private val ioDispatcher: CoroutineDispatcher,
	@MainDispatcher private val mainDispatcher: CoroutineDispatcher,
) : ViewModel() {

	private val _nativeLogs = MutableStateFlow<List<LogMessage>>(emptyList())
	val nativeLogs: StateFlow<List<LogMessage>> = _nativeLogs.asStateFlow()

	private val _vpnLogs = MutableStateFlow<List<LogMessage>>(emptyList())
	val vpnLogs: StateFlow<List<LogMessage>> = _vpnLogs.asStateFlow()

	init {
		viewModelScope.launch(ioDispatcher) {
			logReader.bufferedLogsNative
				.chunked(200, Duration.ofMillis(500))
				.collectLatest { logsChunk ->
					withContext(mainDispatcher) {
						val updated = (_nativeLogs.value + logsChunk)
							.takeLast(Constants.LOG_BUFFER_SIZE.toInt())
						_nativeLogs.value = updated
					}
				}
		}
		viewModelScope.launch(ioDispatcher) {
			logReader.bufferedLogsVPN
				.chunked(200, Duration.ofMillis(500))
				.collectLatest { logsChunk ->
					withContext(mainDispatcher) {
						val updated = (_vpnLogs.value + logsChunk)
							.takeLast(Constants.LOG_BUFFER_SIZE.toInt())
						_vpnLogs.value = updated
					}
				}
		}
	}

	fun shareLogs(context: Context): Job = viewModelScope.launch(ioDispatcher) {
		runCatching {
			val sharePath = File(context.filesDir, "external_files")
			if (sharePath.exists()) sharePath.delete()
			sharePath.mkdir()
			val file = File("${sharePath.path + "/" + Constants.BASE_LOG_FILE_NAME}-${Instant.now().epochSecond}.zip")
			if (file.exists()) file.delete()
			file.createNewFile()
			logReader.zipLogFiles(file.absolutePath)
			val uri = FileProvider.getUriForFile(context, context.getString(R.string.provider), file)
			context.launchShareFile(uri)
		}.onFailure {
			Timber.e(it)
		}
	}

	fun deleteLogs() = viewModelScope.launch {
		logReader.deleteAndClearLogs()
		_nativeLogs.value = emptyList()
		_vpnLogs.value = emptyList()
	}
}
