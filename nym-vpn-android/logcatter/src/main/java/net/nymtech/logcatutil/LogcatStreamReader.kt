package net.nymtech.logcatutil

import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.flow
import kotlinx.coroutines.flow.flowOn
import net.nymtech.logcatutil.model.LogMessage
import timber.log.Timber
import java.io.BufferedReader
import java.io.IOException
import java.io.InputStreamReader

class LogcatStreamReader(
	private val pid: Int,
	private val fileManager: LogFileManager,
) {
	companion object {
		private const val TAG = "logcat-reader"
	}

	private val bufferSize = 1024
	private var process: Process? = null
	private var reader: BufferedReader? = null

	private fun buildCommand(): Array<String> = arrayOf("logcat", "--pid=$pid", "-v", "threadtime")

	private val clearCommand = arrayOf("logcat", "-c")

	@Suppress("MemberVisibilityCanBePrivate")
	var fallbackToTimber: Boolean = false
		private set

	private val ioDispatcher = Dispatchers.IO

	fun readLogs(): Flow<LogMessage> = flow {
		runCatching { clearLogs() }
			.onFailure { Timber.tag(TAG).w(it, "LogcatClearFailed") }

		try {
			process = Runtime.getRuntime().exec(buildCommand())
			reader = BufferedReader(InputStreamReader(process!!.inputStream), bufferSize)

			reader!!.lineSequence().forEach { line ->
				if (line.isNotBlank()) {
					fileManager.writeLog(line)
					emit(LogMessage.from(line))
				}
			}

			Timber.tag(TAG).d("LogcatStreamEnded")
		} catch (e: IOException) {
			Timber.tag(TAG).w(e, "LogcatStreamFailedFallbackToTimber")
			fallbackToTimber = true
			emitFallbackLogs { emit(it) }
		} finally {
			stop()
		}
	}.flowOn(ioDispatcher)

	private suspend fun emitFallbackLogs(emit: suspend (LogMessage) -> Unit = {}) {
		val log = LogMessage.system("Logcat is not accessible. Falling back to Timber logs")
		fileManager.writeLog(log.toString())
		emit(log)
	}

	fun stop() {
		runCatching { process?.destroy() }
			.onFailure { Timber.tag(TAG).w(it, "LogcatProcessDestroyFailed") }

		runCatching { reader?.close() }
			.onFailure { Timber.tag(TAG).w(it, "LogcatReaderCloseFailed") }

		process = null
		reader = null
	}

	fun clearLogs() {
		try {
			Runtime.getRuntime().exec(clearCommand)
		} catch (e: IOException) {
			Timber.tag(TAG).d(e, "LogcatClearBlocked")
		}
	}
}
