package net.nymtech.logcatutil

import kotlinx.coroutines.flow.Flow
import net.nymtech.logcatutil.model.LogMessage

interface LogReader {
	fun start()
	fun stop()
	fun zipLogFiles(path: String)
	suspend fun deleteAndClearLogs()
	val bufferedLogs: Flow<LogMessage>
	val liveLogs: Flow<LogMessage>
}
