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

	/** Writes a line into the app log stream directly, bypassing logcat capture (survives `logcat -c`). */
	suspend fun writeDiagnostic(tag: String, message: String)
	suspend fun zipLogFiles(path: String)

	suspend fun deleteAndClearLogs()

	suspend fun downloadFile(resolver: ContentResolver, uri: Uri, temp: File)
}
