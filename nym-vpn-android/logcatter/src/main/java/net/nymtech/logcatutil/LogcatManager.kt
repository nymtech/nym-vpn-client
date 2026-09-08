package net.nymtech.logcatutil

import android.content.ContentResolver
import android.net.Uri
import androidx.lifecycle.DefaultLifecycleObserver
import androidx.lifecycle.LifecycleOwner
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.Job
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.cancel
import kotlinx.coroutines.channels.BufferOverflow
import kotlinx.coroutines.isActive
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.asSharedFlow
import kotlinx.coroutines.launch
import net.nymtech.logcatutil.model.LogLevel
import net.nymtech.logcatutil.model.LogMessage
import net.nymtech.logcatutil.model.LogType
import timber.log.Timber
import java.io.File
import java.time.Instant

class LogcatManager(private val pid: Int, logDir: String, maxFileSize: Long, maxFolderSize: Long) :
	LogReader,
	DefaultLifecycleObserver {

	companion object {
		private const val TAG = "logcat-manager"
	}

	private val logScope = CoroutineScope(Dispatchers.IO + SupervisorJob())
	private val fileManager = LogFileManager(logDir, maxFileSize, maxFolderSize)

	private val logcatReader = LogcatStreamReader(pid, fileManager)

	private var logJob: Job? = null
	private var isStarted = false
	private var fallbackTree: TimberFileTree? = null

	private val tunnelTags = setOf("core-backend", "core-vpn")
	private fun isLibrary(tag: String): Boolean = tag.contains("libnymvpn", ignoreCase = true)
	private fun isTunnel(tag: String): Boolean = tag in tunnelTags

	private fun classify(tag: String): LogType = when {
		isLibrary(tag) -> LogType.CORE
		isTunnel(tag) -> LogType.TUNNEL
		else -> LogType.APP
	}

	private val _bufferedLogsApp = MutableSharedFlow<LogMessage>(
		replay = 10_000,
		onBufferOverflow = BufferOverflow.DROP_OLDEST,
	)
	private val _bufferedLogsTunnel = MutableSharedFlow<LogMessage>(
		replay = 10_000,
		onBufferOverflow = BufferOverflow.DROP_OLDEST,
	)
	private val _bufferedLogsLibrary = MutableSharedFlow<LogMessage>(
		replay = 10_000,
		onBufferOverflow = BufferOverflow.DROP_OLDEST,
	)

	override val bufferedLogsApp: Flow<LogMessage> = _bufferedLogsApp.asSharedFlow()
	override val bufferedLogsTunnel: Flow<LogMessage> = _bufferedLogsTunnel.asSharedFlow()
	override val bufferedLogsLibrary: Flow<LogMessage> = _bufferedLogsLibrary.asSharedFlow()

	override fun onDestroy(owner: LifecycleOwner) {
		stop()
		logScope.cancel()
	}

	override fun start() {
		if (isStarted) return

		stop()

		Timber.tag(TAG).i("LogcatStart")

		logJob = logScope.launch {
			runCatching {
				logcatReader.readLogs().collect { logMessage ->
					when (classify(logMessage.tag)) {
						LogType.CORE -> _bufferedLogsLibrary.emit(logMessage)
						LogType.TUNNEL -> _bufferedLogsTunnel.emit(logMessage)
						LogType.APP -> _bufferedLogsApp.emit(logMessage)
						LogType.LOGCAT -> Unit
					}
				}
			}.onFailure { t ->
				if (t is CancellationException) {
					Timber.tag(TAG).d("LogcatCollectCancelled")
				} else {
					Timber.tag(TAG).e(t, "LogcatCollectFailed")
				}
			}

			if (isActive && logcatReader.logcatBlocked) {
				activateTimberFallback()
			}
		}

		isStarted = true
	}

	override suspend fun writeDiagnostic(tag: String, message: String) {
		val logMessage = LogMessage(
			time = Instant.now().toString(),
			epochMillis = System.currentTimeMillis(),
			pid = pid.toString(),
			tid = pid.toString(),
			level = LogLevel.INFO,
			tag = tag,
			message = message,
		)
		_bufferedLogsApp.emit(logMessage)
		runCatching { fileManager.writeLog(LogType.LOGCAT, logMessage.toString()) }
			.onFailure { Timber.tag(TAG).w(it, "DiagnosticWriteFailed") }
		runCatching { fileManager.writeLog(LogType.APP, logMessage.toString()) }
			.onFailure { Timber.tag(TAG).w(it, "DiagnosticWriteFailed") }
	}

	private fun activateTimberFallback() {
		if (fallbackTree != null) return
		Timber.tag(TAG).w("TimberFallbackActivated")
		val tree = TimberFileTree(fileManager, logScope)
		fallbackTree = tree
		Timber.plant(tree)
	}

	override fun stop() {
		if (!isStarted) return

		Timber.tag(TAG).i("LogcatStop")

		fallbackTree?.let {
			Timber.uproot(it)
			fallbackTree = null
		}

		runCatching { logJob?.cancel() }
			.onFailure { Timber.tag(TAG).w(it, "LogcatJobCancelFailed") }

		runCatching { logcatReader.stop() }
			.onFailure { Timber.tag(TAG).w(it, "LogcatReaderStopFailed") }

		runCatching { fileManager.close() }
			.onFailure { Timber.tag(TAG).w(it, "LogcatFileManagerCloseFailed") }

		logJob = null
		isStarted = false
	}

	override suspend fun zipLogFiles(path: String) {
		Timber.tag(TAG).i("LogsZipRequested")

		val wasStarted = isStarted
		stop()

		runCatching {
			fileManager.zipLogs(path)
		}.onFailure { t ->
			Timber.tag(TAG).e(t, "LogsZipFailed")
			throw t
		}.onSuccess {
			Timber.tag(TAG).i("LogsZipSuccess")
		}

		if (wasStarted) {
			runCatching { logcatReader.clearLogs() }
				.onFailure { Timber.tag(TAG).w(it, "LogcatClearFailedAfterZip") }
			start()
		}
	}

	@OptIn(ExperimentalCoroutinesApi::class)
	override suspend fun deleteAndClearLogs() {
		Timber.tag(TAG).i("LogsDeleteRequested")

		val wasStarted = isStarted
		stop()

		runCatching {
			_bufferedLogsApp.resetReplayCache()
			_bufferedLogsTunnel.resetReplayCache()
			_bufferedLogsLibrary.resetReplayCache()
			fileManager.deleteAllLogs()
		}.onFailure { t ->
			Timber.tag(TAG).e(t, "LogsDeleteFailed")
		}.onSuccess {
			Timber.tag(TAG).i("LogsDeleteSuccess")
		}

		if (wasStarted) start()
	}

	override suspend fun downloadFile(resolver: ContentResolver, uri: Uri, temp: File) {
		Timber.tag(TAG).i("LogsDownloadToUriRequested")

		val wasStarted = isStarted
		stop()

		runCatching {
			fileManager.zipLogs(temp.absolutePath)

			resolver.openOutputStream(uri).use { outputStream ->
				if (outputStream == null) throw IllegalStateException("Failed to get output stream")
				temp.inputStream().use { inputStream ->
					inputStream.copyTo(outputStream)
				}
			}
		}.onFailure { t ->
			Timber.tag(TAG).e(t, "LogsDownloadToUriFailed")
			throw t
		}.onSuccess {
			Timber.tag(TAG).i("LogsDownloadToUriSuccess")
		}

		if (wasStarted) {
			runCatching { logcatReader.clearLogs() }
				.onFailure { Timber.tag(TAG).w(it, "LogcatClearFailedAfterDownload") }
			start()
		}
	}
}
