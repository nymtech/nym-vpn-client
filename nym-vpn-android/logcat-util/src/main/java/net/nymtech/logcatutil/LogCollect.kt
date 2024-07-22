package net.nymtech.logcatutil

import kotlinx.coroutines.flow.Flow
import net.nymtech.logcatutil.model.LogMessage
import java.io.File

interface LogCollect {
	fun start(onLogMessage: ((message: LogMessage) -> Unit)? = null)
	fun stop()
	suspend fun getLogFile(): Result<File>
	val bufferedLogs: Flow<LogMessage>
	val liveLogs: Flow<LogMessage>
}
