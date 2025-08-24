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
	private val bufferSize = 1024
	private var process: Process? = null
	private var reader: BufferedReader? = null
	private val command = "logcat -v epoch | grep \"($pid)\""
	private val clearCommand = "logcat -c"

	private var fallbackToTimber = false

	private val ioDispatcher = Dispatchers.IO

	fun readLogs(): Flow<LogMessage> = flow {
		try {
			clearLogs()
			process = Runtime.getRuntime().exec(command)
			reader = BufferedReader(InputStreamReader(process!!.inputStream), bufferSize)
			reader!!.lineSequence().forEach { line ->
				if (line.isNotEmpty()) {
					fileManager.writeLog(line)
					emit(LogMessage.from(line))
				}
			}
		} catch (e: IOException) {
			Timber.e(e, "LogcatStreamReader failed, fallback to Timber")
			fallbackToTimber = true
			emitFallbackLogs { emit(it) }
		} finally {
			stop()
		}
	}.flowOn(ioDispatcher)

	private suspend fun emitFallbackLogs(emit: suspend (LogMessage) -> Unit = {}) {
		val fallbackMessage = "Logcat is not accessible. Falling back to Timber logs"
		val log = LogMessage.system(fallbackMessage)
		fileManager.writeLog(log.toString())
		emit(log)
	}

	fun start() {
		if (process == null && !fallbackToTimber) {
			try {
				process = Runtime.getRuntime().exec(command)
				reader = BufferedReader(InputStreamReader(process!!.inputStream), bufferSize)
			} catch (e: IOException) {
				Timber.e(e, "Failed to start logcat process")
				fallbackToTimber = true
			}
		}
	}

	fun stop() {
		process?.destroy()
		reader?.close()
		process = null
		reader = null
	}

	fun clearLogs() {
		try {
			Runtime.getRuntime().exec(clearCommand)
		} catch (e: IOException) {
			Timber.w(e, "Could not clear logcat logs")
		}
	}
}
