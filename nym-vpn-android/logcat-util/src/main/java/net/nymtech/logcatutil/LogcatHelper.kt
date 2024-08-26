package net.nymtech.logcatutil

import android.content.Context
import android.os.Build
import androidx.annotation.RequiresApi
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.channels.BufferOverflow
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.asSharedFlow
import kotlinx.coroutines.withContext
import net.nymtech.logcatutil.model.LogMessage
import timber.log.Timber
import java.io.BufferedOutputStream
import java.io.BufferedReader
import java.io.File
import java.io.FileNotFoundException
import java.io.FileOutputStream
import java.io.FileReader
import java.io.IOException
import java.io.InputStreamReader
import java.io.PrintWriter
import java.nio.file.Files
import java.nio.file.Paths
import java.nio.file.StandardOpenOption
import java.util.zip.ZipEntry
import java.util.zip.ZipOutputStream

object LogcatHelper {

	private const val MAX_FILE_SIZE = 2097152L // 2MB
	private const val MAX_FOLDER_SIZE = 10485760L // 10MB

	private val ioDispatcher = Dispatchers.IO

	private object LogcatHelperInit {
		var maxFileSize: Long = MAX_FILE_SIZE
		var maxFolderSize: Long = MAX_FOLDER_SIZE
		var pID: Int = 0
		var publicAppDirectory = ""
		var logcatPath = ""
	}

	fun init(maxFileSize: Long = MAX_FILE_SIZE, maxFolderSize: Long = MAX_FOLDER_SIZE, context: Context): LogCollect {
		if (maxFileSize > maxFolderSize) {
			throw IllegalStateException("maxFileSize must be less than maxFolderSize")
		}
		synchronized(LogcatHelperInit) {
			LogcatHelperInit.maxFileSize = maxFileSize
			LogcatHelperInit.maxFolderSize = maxFolderSize
			LogcatHelperInit.pID = android.os.Process.myPid()
			context.getExternalFilesDir(null)?.let {
				LogcatHelperInit.publicAppDirectory = it.absolutePath
				LogcatHelperInit.logcatPath = LogcatHelperInit.publicAppDirectory + File.separator + "logs"
				val logDirectory = File(LogcatHelperInit.logcatPath)
				if (!logDirectory.exists()) {
					logDirectory.mkdir()
				}
			}
			return Logcat
		}
	}

	internal object Logcat : LogCollect {

		private var logcatReader: LogcatReader? = null

		override suspend fun start(onLogMessage: ((message: LogMessage) -> Unit)?) {
			withContext(ioDispatcher) {
				logcatReader ?: run {
					logcatReader = LogcatReader(LogcatHelperInit.pID.toString(), LogcatHelperInit.logcatPath, onLogMessage)
				}
				logcatReader?.run()
			}
		}

		override fun stop() {
			logcatReader?.stopLogs()
			logcatReader = null
		}

		override suspend fun zipLogFiles(path: String) {
			return withContext(ioDispatcher) {
					stop()
					zipAll(path)
				}.also {
				start()
			}
		}

		private fun zipAll(zipFilePath: String) {
			val sourceFile = File(LogcatHelperInit.logcatPath)
			val outputZipFile = File(zipFilePath)
			ZipOutputStream(BufferedOutputStream(FileOutputStream(outputZipFile))).use { zos ->
				sourceFile.walkTopDown().forEach { file ->
					val zipFileName = file.absolutePath.removePrefix(sourceFile.absolutePath).removePrefix("/")
					val entry = ZipEntry("$zipFileName${(if (file.isDirectory) "/" else "")}")
					zos.putNextEntry(entry)
					if (file.isFile) {
						file.inputStream().copyTo(zos)
					}
				}
			}
		}

		@OptIn(ExperimentalCoroutinesApi::class)
		override suspend fun deleteAndClearLogs() {
			withContext(ioDispatcher) {
				_bufferedLogs.resetReplayCache()
				logcatReader?.deleteAllFiles()
			}
		}

		private val _bufferedLogs = MutableSharedFlow<LogMessage>(
			replay = 10_000,
			onBufferOverflow = BufferOverflow.DROP_OLDEST,
		)
		private val _liveLogs = MutableSharedFlow<LogMessage>(
			replay = 1,
			onBufferOverflow = BufferOverflow.DROP_OLDEST,
		)

		override val bufferedLogs: Flow<LogMessage> = _bufferedLogs.asSharedFlow()

		override val liveLogs: Flow<LogMessage> = _liveLogs.asSharedFlow()

		private class LogcatReader(
			pID: String,
			private val logcatPath: String,
			private val callback: ((input: LogMessage) -> Unit)?,
		) {
			private var logcatProc: Process? = null
			private var reader: BufferedReader? = null
			private var mRunning = true
			private var command = ""
			private var clearLogCommand = ""
			private var outputStream: FileOutputStream? = null

			init {
				try {
					outputStream = FileOutputStream(createLogFile(logcatPath))
				} catch (e: FileNotFoundException) {
					Timber.e(e)
				}

				command = "logcat -v epoch | grep \"($pID)\""
				clearLogCommand = "logcat -c"
			}

			fun stopLogs() {
				mRunning = false
			}

			fun clear() {
				Runtime.getRuntime().exec(clearLogCommand)
			}

			fun run() {
				if (outputStream == null) return
				try {
					clear()
					logcatProc = Runtime.getRuntime().exec(command)
					reader = BufferedReader(InputStreamReader(logcatProc!!.inputStream), 1024)
					var line: String? = null

					while (mRunning && run {
							line = reader!!.readLine()
							line
						} != null
					) {
						if (!mRunning) {
							break
						}
						if (line!!.isEmpty()) {
							continue
						}

						if (outputStream!!.channel.size() >= LogcatHelperInit.maxFileSize) {
							outputStream!!.close()
							outputStream = FileOutputStream(createLogFile(logcatPath))
						}
						if (getFolderSize(logcatPath) >= LogcatHelperInit.maxFolderSize) {
							deleteOldestFile()
						}
						line?.let { text ->
							outputStream!!.write((text + System.lineSeparator()).toByteArray())
							try {
								val logMessage = LogMessage.from(text)
								_bufferedLogs.tryEmit(logMessage)
								_liveLogs.tryEmit(logMessage)
								callback?.let {
									it(logMessage)
								}
							} catch (e: Exception) {
								Timber.e(e)
							}
						}
					}
				} catch (e: IOException) {
					Timber.e(e)
				} finally {
					logcatProc?.destroy()
					logcatProc = null

					try {
						reader?.close()
						outputStream?.close()
						reader = null
						outputStream = null
					} catch (e: IOException) {
						Timber.e(e)
					}
				}
			}

			private fun getFolderSize(path: String): Long {
				File(path).run {
					var size = 0L
					if (this.isDirectory && this.listFiles() != null) {
						for (file in this.listFiles()!!) {
							size += getFolderSize(file.absolutePath)
						}
					} else {
						size = this.length()
					}
					return size
				}
			}

			private fun createLogFile(dir: String): File {
				return File(dir, "logcat_" + System.currentTimeMillis() + ".txt")
			}

			fun deleteOldestFile() {
				val directory = File(logcatPath)
				if (directory.isDirectory) {
					directory.listFiles()?.toMutableList()?.run {
						this.sortBy { it.lastModified() }
						this.first().delete()
					}
				}
			}
			fun deleteAllFiles() {
				val directory = File(logcatPath)
				directory.listFiles()?.toMutableList()?.run {
					this.forEach { it.delete() }
				}
			}
		}
	}
}
