package net.nymtech.logcatutil

import android.content.ContentResolver
import android.net.Uri
import kotlinx.coroutines.flow.Flow
import net.nymtech.logcatutil.model.LogMessage
import java.io.File

interface LogReader {
	val bufferedLogsApp: Flow<LogMessage>
	val bufferedLogsTunnel: Flow<LogMessage>
	val bufferedLogsLibrary: Flow<LogMessage>

	fun start()
	fun stop()
	suspend fun zipLogFiles(path: String)

	suspend fun deleteAndClearLogs()

	suspend fun downloadFile(resolver: ContentResolver, uri: Uri, temp: File)
}
