package net.nymtech.logcatutil

import android.os.StrictMode
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock
import kotlinx.coroutines.withContext
import net.nymtech.logcatutil.model.LogType
import java.io.BufferedOutputStream
import java.io.File
import java.io.FileOutputStream
import java.util.zip.ZipEntry
import java.util.zip.ZipOutputStream

class LogFileManager(private val logDir: String, private val maxFileSize: Long, private val maxFolderSize: Long) {
	private val ioDispatcher = Dispatchers.IO
	private val mutex = Mutex()

	private data class WriterState(var currentFile: File? = null, var outputStream: BufferedOutputStream? = null, var bytesWritten: Long = 0L)

	private val writers: MutableMap<LogType, WriterState> = mutableMapOf(
		LogType.APP to WriterState(),
		LogType.TUNNEL to WriterState(),
		LogType.CORE to WriterState(),
		LogType.LOGCAT to WriterState(),
	)

	suspend fun writeLog(type: LogType, line: String) = withContext(ioDispatcher) {
		val oldPolicy = StrictMode.allowThreadDiskWrites()
		try {
			mutex.withLock {
				rotateIfNeededLocked(type)

				val state = writers.getValue(type)
				try {
					val bytes = (line + System.lineSeparator()).toByteArray()
					state.outputStream?.write(bytes)
					state.outputStream?.flush()
					state.bytesWritten += bytes.size
				} catch (_: Exception) {
					// ignore
				}
			}
		} finally {
			StrictMode.setThreadPolicy(oldPolicy)
		}
	}

	suspend fun writeLog(line: String) = writeLog(LogType.APP, line)

	suspend fun zipLogs(zipFilePath: String) = withContext(ioDispatcher) {
		val oldPolicy = StrictMode.allowThreadDiskWrites()
		try {
			mutex.withLock {
				closeAllLocked()

				val sourceDir = File(logDir)
				if (!sourceDir.exists() || !sourceDir.isDirectory) return@withLock

				val outputZipFile = File(zipFilePath)
				ZipOutputStream(BufferedOutputStream(FileOutputStream(outputZipFile))).use { zos ->
					sourceDir.listFiles()
						?.asSequence()
						?.filter { it.exists() && it.isFile }
						?.filter { it.length() > 0L }
						?.sortedBy { it.lastModified() }
						?.forEach { file ->
							val folder = folderForZip(file.name)
							val entryName = "$folder/${file.name}"
							zos.putNextEntry(ZipEntry(entryName))
							file.inputStream().use { it.copyTo(zos) }
							zos.closeEntry()
						}
				}
			}
		} finally {
			StrictMode.setThreadPolicy(oldPolicy)
		}
	}

	private fun folderForZip(fileName: String): String = when {
		fileName.startsWith("app_", ignoreCase = true) -> "app"
		fileName.startsWith("tunnel_", ignoreCase = true) -> "tunnel"
		fileName.startsWith("core_", ignoreCase = true) -> "core"
		fileName.startsWith("logcat_", ignoreCase = true) -> "raw"
		else -> "other"
	}

	suspend fun deleteAllLogs() = withContext(ioDispatcher) {
		val oldPolicy = StrictMode.allowThreadDiskWrites()
		try {
			mutex.withLock {
				closeAllLocked()
				File(logDir).listFiles()?.forEach { it.deleteRecursively() }
			}
		} finally {
			StrictMode.setThreadPolicy(oldPolicy)
		}
	}

	fun close() {
		runCatching {
			writers.values.forEach { st ->
				try {
					st.outputStream?.close()
				} catch (_: Exception) {}
				st.outputStream = null
				st.currentFile = null
				st.bytesWritten = 0
			}
		}
	}

	private fun closeAllLocked() {
		writers.values.forEach { st ->
			try {
				st.outputStream?.close()
			} catch (_: Exception) {}
			st.outputStream = null
			st.currentFile = null
			st.bytesWritten = 0
		}
	}

	private fun rotateIfNeededLocked(type: LogType) {
		val state = writers.getValue(type)
		val needsRotate = state.currentFile == null ||
			state.outputStream == null ||
			state.bytesWritten >= maxFileSize

		if (needsRotate) {
			try {
				state.outputStream?.close()
			} catch (_: Exception) {}
			state.outputStream = null
			state.bytesWritten = 0

			val dir = File(logDir)
			if (!dir.exists()) {
				dir.mkdirs()
			}

			enforceFolderLimitLocked()

			val file = File(logDir, "${type.prefix}_${System.currentTimeMillis()}.txt")
			state.currentFile = file
			try {
				state.outputStream = BufferedOutputStream(FileOutputStream(file, true))
			} catch (e: Exception) {
				state.currentFile = null
				state.outputStream = null
			}
		}
	}

	private fun enforceFolderLimitLocked() {
		val dir = File(logDir)
		if (!dir.exists()) return

		var folderSize = getFolderSize(dir)
		while (folderSize >= maxFolderSize) {
			val oldest = dir.listFiles()
				?.filter { it.isFile }
				?.minByOrNull { it.lastModified() }
				?: break

			val openedEntry = writers.entries.find { it.value.currentFile?.absolutePath == oldest.absolutePath }
			if (openedEntry != null) {
				val st = openedEntry.value
				try {
					st.outputStream?.close()
				} catch (_: Exception) {}
				st.outputStream = null
				st.currentFile = null
				st.bytesWritten = 0
			}

			oldest.delete()
			folderSize = getFolderSize(dir)
		}
	}

	private fun getFolderSize(dir: File): Long {
		var size = 0L
		dir.listFiles()?.forEach { f ->
			size += if (f.isDirectory) getFolderSize(f) else f.length()
		}
		return size
	}
}
