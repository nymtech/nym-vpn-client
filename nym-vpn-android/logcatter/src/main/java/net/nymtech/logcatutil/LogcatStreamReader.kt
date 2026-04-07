package net.nymtech.logcatutil

import android.os.StrictMode
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.flow
import kotlinx.coroutines.flow.flowOn
import net.nymtech.logcatutil.model.LogMessage
import net.nymtech.logcatutil.model.LogType
import timber.log.Timber
import java.io.BufferedReader
import java.io.IOException
import java.io.InputStreamReader
import kotlin.math.abs

class LogcatStreamReader(private val pid: Int, private val fileManager: LogFileManager) {
	companion object {
		private const val TAG = "logcat-reader"
		private const val MERGE_WINDOW_MS = 250L
	}

	private val bufferSize = 8192
	private var process: Process? = null
	private var reader: BufferedReader? = null

	private fun buildCommand(): Array<String> = arrayOf("logcat", "--pid=$pid", "-v", "threadtime")
	private val clearCommand = arrayOf("logcat", "-c")

	@Suppress("MemberVisibilityCanBePrivate")
	var fallbackToTimber: Boolean = false
		private set

	var logcatBlocked: Boolean = false
		private set

	private val ioDispatcher = Dispatchers.IO

	private val tunnelTags = setOf("core-backend", "core-vpn")
	private fun isLibrary(tag: String): Boolean = tag.contains("libnymvpn", ignoreCase = true)
	private fun isTunnel(tag: String): Boolean = tag in tunnelTags
	private fun classify(tag: String): LogType = when {
		isLibrary(tag) -> LogType.CORE
		isTunnel(tag) -> LogType.TUNNEL
		else -> LogType.APP
	}

	private fun looksLikeContinuation(msg: String): Boolean {
		if (msg.isEmpty()) return false
		if (msg[0].isWhitespace()) return true

		return msg.startsWith("at ") ||
			msg.startsWith("Caused by:") ||
			msg.startsWith("Suppressed:") ||
			msg.startsWith("{") ||
			msg.startsWith("}") ||
			msg.startsWith("[") ||
			msg.startsWith("]") ||
			msg.startsWith(")") ||
			msg.startsWith("(")
	}

	private fun shouldMerge(pending: LogMessage, next: LogMessage): Boolean {
		if (pending.pid != next.pid) return false
		if (pending.tid != next.tid) return false
		if (pending.tag != next.tag) return false
		if (pending.level != next.level) return false

		val dt = abs(next.epochMillis - pending.epochMillis)
		if (dt > MERGE_WINDOW_MS) return false

		return looksLikeContinuation(next.message)
	}

	private fun appendContinuation(pending: LogMessage, continuationLine: String): LogMessage {
		val extra = continuationLine.trimEnd()
		if (extra.isBlank()) return pending
		val combined = if (pending.message.isBlank()) extra else pending.message + "\n" + extra
		return pending.copy(message = combined)
	}

	private suspend fun flushPending(pending: LogMessage?, emit: suspend (LogMessage) -> Unit) {
		if (pending == null) return
		val type = classify(pending.tag)
		runCatching {
			fileManager.writeLog(LogType.LOGCAT, pending.toString())
			fileManager.writeLog(type, pending.toString())
		}.onFailure { Timber.tag(TAG).w(it, "LogWriteFailed") }
		emit(pending)
	}

	fun readLogs(): Flow<LogMessage> = flow {
		logcatBlocked = false
		val oldPolicy = StrictMode.allowThreadDiskWrites()
		try {
			runCatching { clearLogs() }
				.onFailure { Timber.tag(TAG).w(it, "LogcatClearFailed") }
			var pending: LogMessage? = null
			var linesRead = 0
			try {
				process = Runtime.getRuntime().exec(buildCommand())
				reader = BufferedReader(InputStreamReader(process!!.inputStream), bufferSize)

				reader!!.lineSequence().forEach { raw ->
					linesRead++
					val line = raw.trimEnd()
					if (line.isBlank()) return@forEach
					val parsed = LogMessage.tryFromThreadtime(line)
					if (parsed != null) {
						if (pending != null && shouldMerge(pending!!, parsed)) {
							pending = pending!!.copy(
								message = pending!!.message + "\n" + parsed.message.trimEnd(),
							)
						} else {
							flushPending(pending) { emit(it) }
							pending = parsed
						}
						return@forEach
					}

					pending = if (pending != null) {
						appendContinuation(pending!!, line)
					} else {
						LogMessage.system(line)
					}
				}

				flushPending(pending) { emit(it) }
				if (linesRead == 0) {
					logcatBlocked = true
					Timber.tag(TAG).w("LogcatBlockedFallbackToTimber")
				} else {
					Timber.tag(TAG).d("LogcatStreamEnded")
				}
			} catch (e: IOException) {
				Timber.tag(TAG).w(e, "LogcatStreamFailedFallbackToTimber")
				fallbackToTimber = true
				val log = LogMessage.system("Logcat is not accessible. Falling back to Timber logs")
				runCatching { fileManager.writeLog(LogType.APP, log.toString()) }
					.onFailure { Timber.tag(TAG).w(it, "FallbackLogWriteFailed") }
				emit(log)
			} finally {
				stop()
			}
		} finally {
			StrictMode.setThreadPolicy(oldPolicy)
		}
	}.flowOn(ioDispatcher)

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
