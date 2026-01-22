package net.nymtech.logcatutil

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

class LogFileManager(
	private val logDir: String,
	private val maxFileSize: Long,
	private val maxFolderSize: Long,
) {
	private val ioDispatcher = Dispatchers.IO
	private val mutex = Mutex()

	private data class WriterState(
		var currentFile: File? = null,
		var outputStream: FileOutputStream? = null,
	)

	private val writers: MutableMap<LogType, WriterState> = mutableMapOf(
		LogType.APP to WriterState(),
		LogType.TUNNEL to WriterState(),
		LogType.CORE to WriterState(),
		LogType.LOGCAT to WriterState(),
	)

	init {
		File(logDir).mkdirs()
	}

	suspend fun writeLog(type: LogType, line: String) = withContext(ioDispatcher) {
		mutex.withLock {
			rotateIfNeededLocked(type)

			val state = writers.getValue(type)
			try {
				state.outputStream?.write((line + System.lineSeparator()).toByteArray())
				state.outputStream?.flush()
			} catch (_: Exception) {
				// ignore (or log if you want)
			}
		}
	}

	suspend fun writeLog(line: String) = writeLog(LogType.APP, line)

	suspend fun zipLogs(zipFilePath: String) = withContext(ioDispatcher) {
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
	}

	private fun folderForZip(fileName: String): String = when {
		fileName.startsWith("app_", ignoreCase = true) -> "app"
		fileName.startsWith("tunnel_", ignoreCase = true) -> "tunnel"
		fileName.startsWith("core_", ignoreCase = true) -> "core"
		fileName.startsWith("logcat_", ignoreCase = true) -> "raw"
		else -> "other"
	}

	suspend fun deleteAllLogs() = withContext(ioDispatcher) {
		mutex.withLock {
			closeAllLocked()
			File(logDir).listFiles()?.forEach { it.deleteRecursively() }
			File(logDir).mkdirs()
		}
	}

	fun close() {
		runCatching {
			writers.values.forEach { st ->
				st.outputStream?.close()
				st.outputStream = null
				st.currentFile = null
			}
		}
	}

	private fun closeAllLocked() {
		writers.values.forEach { st ->
			st.outputStream?.close()
			st.outputStream = null
			st.currentFile = null
		}
	}

	private fun rotateIfNeededLocked(type: LogType) {
		enforceFolderLimitLocked()

		val state = writers.getValue(type)
		val currentSize = state.currentFile?.length() ?: 0L
		val needsRotate = state.currentFile == null ||
			state.outputStream == null ||
			currentSize >= maxFileSize

		if (needsRotate) {
			state.outputStream?.close()
			state.outputStream = null

			File(logDir).mkdirs()

			val file = File(logDir, "${type.prefix}_${System.currentTimeMillis()}.txt")
			state.currentFile = file
			state.outputStream = FileOutputStream(file, true)
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

			val opened = writers.values.any { it.currentFile?.absolutePath == oldest.absolutePath }
			if (opened) {
				writers.entries.forEach { (type, st) ->
					if (st.currentFile?.absolutePath == oldest.absolutePath) {
						st.outputStream?.close()
						st.outputStream = null
						st.currentFile = null
						rotateIfNeededLocked(type)
					}
				}
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
