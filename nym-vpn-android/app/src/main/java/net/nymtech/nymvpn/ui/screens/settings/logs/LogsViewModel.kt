package net.nymtech.nymvpn.ui.screens.settings.logs

import androidx.compose.runtime.mutableStateListOf
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import net.nymtech.logcathelper.LogCollect
import net.nymtech.logcathelper.model.LogMessage
import net.nymtech.nymvpn.module.IoDispatcher
import net.nymtech.nymvpn.module.MainDispatcher
import net.nymtech.nymvpn.util.Constants
import net.nymtech.nymvpn.util.FileUtils
import net.nymtech.nymvpn.util.chunked
import java.time.Duration
import java.time.Instant
import javax.inject.Inject

@HiltViewModel
class LogsViewModel @Inject constructor(
	private val logCollect: LogCollect,
	private val fileUtils: FileUtils,
	@IoDispatcher private val ioDispatcher: CoroutineDispatcher,
	@MainDispatcher private val mainDispatcher: CoroutineDispatcher,
) : ViewModel() {

	val logs = mutableStateListOf<LogMessage>()

	init {
		viewModelScope.launch(ioDispatcher) {
			logCollect.bufferedLogs.chunked(500, Duration.ofSeconds(1)).collect {
				withContext(mainDispatcher) {
					logs.addAll(it)
				}
				if (logs.size > Constants.LOG_BUFFER_SIZE) {
					withContext(mainDispatcher) {
						logs.removeRange(0, (logs.size - Constants.LOG_BUFFER_SIZE).toInt())
					}
				}
			}
		}
	}

	suspend fun saveLogsToFile(): Result<Unit> {
		val file = logCollect.getLogFile().getOrElse {
			return Result.failure(it)
		}
		val fileContent = fileUtils.readBytesFromFile(file)
		val fileName = "${Constants.BASE_LOG_FILE_NAME}-${Instant.now().epochSecond}.txt"
		return fileUtils.saveByteArrayToDownloads(fileContent, fileName)
	}
}
