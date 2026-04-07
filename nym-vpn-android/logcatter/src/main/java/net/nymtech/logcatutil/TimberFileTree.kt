package net.nymtech.logcatutil

import android.util.Log
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.launch
import net.nymtech.logcatutil.model.LogLevel
import net.nymtech.logcatutil.model.LogMessage
import net.nymtech.logcatutil.model.LogType
import timber.log.Timber
import java.time.Instant

class TimberFileTree(private val fileManager: LogFileManager, private val scope: CoroutineScope) : Timber.Tree() {

	private val tunnelTags = setOf("core-backend", "core-vpn")

	private fun isLibrary(tag: String): Boolean = tag.contains("libnymvpn", ignoreCase = true)
	private fun isTunnel(tag: String): Boolean = tag in tunnelTags
	private fun classify(tag: String): LogType = when {
		isLibrary(tag) -> LogType.CORE
		isTunnel(tag) -> LogType.TUNNEL
		else -> LogType.APP
	}

	private fun Int.toLogLevel(): LogLevel = when (this) {
		Log.VERBOSE -> LogLevel.VERBOSE
		Log.DEBUG -> LogLevel.DEBUG
		Log.INFO -> LogLevel.INFO
		Log.WARN -> LogLevel.WARNING
		Log.ERROR -> LogLevel.ERROR
		Log.ASSERT -> LogLevel.ASSERT
		else -> LogLevel.INFO
	}

	override fun log(priority: Int, tag: String?, message: String, t: Throwable?) {
		val resolvedTag = tag ?: "App"
		val type = classify(resolvedTag)
		val logMessage = LogMessage(
			time = Instant.now().toString(),
			epochMillis = System.currentTimeMillis(),
			pid = android.os.Process.myPid().toString(),
			tid = android.os.Process.myTid().toString(),
			level = priority.toLogLevel(),
			tag = resolvedTag,
			message = message,
		)
		val line = logMessage.toString()
		scope.launch {
			runCatching {
				fileManager.writeLog(LogType.LOGCAT, line)
				fileManager.writeLog(type, line)
			}
		}
	}
}
