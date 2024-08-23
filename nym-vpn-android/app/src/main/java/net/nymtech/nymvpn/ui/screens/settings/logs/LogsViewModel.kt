package net.nymtech.nymvpn.ui.screens.settings.logs

import android.content.Context
import androidx.compose.runtime.mutableStateListOf
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import net.nymtech.logcatutil.LogCollect
import net.nymtech.logcatutil.model.LogMessage
import net.nymtech.nymvpn.module.IoDispatcher
import net.nymtech.nymvpn.module.MainDispatcher
import net.nymtech.nymvpn.util.Constants
import net.nymtech.nymvpn.util.extensions.chunked
import net.nymtech.nymvpn.util.extensions.shareFile
import java.time.Duration
import java.time.Instant
import javax.inject.Inject

@HiltViewModel
class LogsViewModel @Inject constructor(
	private val logCollect: LogCollect,
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

	fun shareLogs(context: Context) = viewModelScope.launch(ioDispatcher) {
		val fileName = "${Constants.BASE_LOG_FILE_NAME}-${Instant.now().epochSecond}.txt"
		val file = logCollect.getLogFile(fileName).getOrElse {
			// TODO add error message
			return@launch
		}
		context.shareFile(file)
	}

	fun deleteLogs() = viewModelScope.launch {
		logCollect.deleteAndClearLogs()
		logs.clear()
	}
}
