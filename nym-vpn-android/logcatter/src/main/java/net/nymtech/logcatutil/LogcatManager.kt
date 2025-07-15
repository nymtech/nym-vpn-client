package net.nymtech.logcatutil

import androidx.lifecycle.DefaultLifecycleObserver
import androidx.lifecycle.LifecycleOwner
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.Job
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.cancel
import kotlinx.coroutines.channels.BufferOverflow
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.asSharedFlow
import kotlinx.coroutines.launch
import net.nymtech.logcatutil.model.LogMessage

class LogcatManager(
	pid: Int,
	logDir: String,
	maxFileSize: Long,
	maxFolderSize: Long,
) : LogReader, DefaultLifecycleObserver {
	private val logScope = CoroutineScope(Dispatchers.IO + SupervisorJob())
	private val fileManager = LogFileManager(logDir, maxFileSize, maxFolderSize)
	private val logcatReader = LogcatStreamReader(pid, fileManager)
	private var logJob: Job? = null
	private var isStarted = false

	private val _bufferedLogsNative = MutableSharedFlow<LogMessage>(
		replay = 10_000,
		onBufferOverflow = BufferOverflow.DROP_OLDEST,
	)

	private val _bufferedLogsVPN = MutableSharedFlow<LogMessage>(
		replay = 10_000,
		onBufferOverflow = BufferOverflow.DROP_OLDEST,
	)

	override val bufferedLogsNative: Flow<LogMessage> = _bufferedLogsNative.asSharedFlow()
	override val bufferedLogsVPN: Flow<LogMessage> = _bufferedLogsVPN.asSharedFlow()

	override fun onCreate(owner: LifecycleOwner) {
		// for auto start
		// start()
	}

	override fun onDestroy(owner: LifecycleOwner) {
		stop()
		logScope.cancel()
	}

	override fun start() {
		if (isStarted) return
		stop()
		logJob = logScope.launch {
			logcatReader.readLogs().collect { logMessage ->
				if (logMessage.tag.contains("libnymvpn")) {
					_bufferedLogsVPN.emit(logMessage)
				} else {
					_bufferedLogsNative.emit(logMessage)
				}
			}
		}
		isStarted = true
	}

	override fun stop() {
		if (!isStarted) return
		logJob?.cancel()
		logcatReader.stop()
		fileManager.close()
		isStarted = false
	}

	override fun zipLogFiles(path: String) {
		logScope.launch {
			val wasStarted = isStarted
			stop()
			fileManager.zipLogs(path)
			if (wasStarted) {
				logcatReader.clearLogs()
				start()
			}
		}
	}

	@OptIn(ExperimentalCoroutinesApi::class)
	override suspend fun deleteAndClearLogs() {
		val wasStarted = isStarted
		stop()
		_bufferedLogsVPN.resetReplayCache()
		_bufferedLogsNative.resetReplayCache()
		fileManager.deleteAllLogs()
		if (wasStarted) start()
	}
}
