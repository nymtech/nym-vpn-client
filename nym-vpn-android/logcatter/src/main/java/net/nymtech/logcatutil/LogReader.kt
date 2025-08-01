package net.nymtech.logcatutil

import android.content.ContentResolver
import android.net.Uri
import kotlinx.coroutines.flow.Flow
import net.nymtech.logcatutil.model.LogMessage
import java.io.File

interface LogReader {
	fun start()
	fun stop()
	fun zipLogFiles(path: String)
	suspend fun deleteAndClearLogs()
	suspend fun downloadFile(resolver: ContentResolver, uri: Uri, temp: File)
	val bufferedLogsNative: Flow<LogMessage>
	val bufferedLogsVPN: Flow<LogMessage>
}
