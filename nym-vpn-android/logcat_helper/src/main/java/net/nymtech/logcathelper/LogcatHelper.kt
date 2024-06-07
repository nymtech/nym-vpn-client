package net.nymtech.logcathelper

import android.content.Context
import android.os.Build
import androidx.annotation.RequiresApi
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.channels.BufferOverflow
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.asSharedFlow
import kotlinx.coroutines.withContext
import net.nymtech.logcathelper.model.LogMessage
import timber.log.Timber
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

object LogcatHelper {

	private const val MAX_FILE_SIZE = 2097152L // 2MB
	private const val MAX_FOLDER_SIZE = 10485760L // 10MB

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

		override fun start(onLogMessage: ((message: LogMessage) -> Unit)?) {
			logcatReader ?: run {
				logcatReader = LogcatReader(LogcatHelperInit.pID.toString(), LogcatHelperInit.logcatPath, onLogMessage)
			}
			logcatReader?.let { logReader ->
				if (!logReader.isAlive) logReader.start()
			}
		}

		override fun stop() {
			logcatReader?.stopLogs()
			logcatReader = null
		}

		private fun mergeLogs(sourceDir: String, outputFile: File) {
			val logcatDir = File(sourceDir)

			if (!outputFile.exists()) outputFile.createNewFile()
			val pw = PrintWriter(outputFile)
			val logFiles = logcatDir.listFiles()

			logFiles?.sortBy { it.lastModified() }

			logFiles?.forEach { logFile ->
				val br = BufferedReader(FileReader(logFile))

				var line: String?
				while (run {
						line = br.readLine()
						line
					} != null
				) {
					pw.println(line)
				}
			}
			pw.flush()
			pw.close()
		}

		@RequiresApi(Build.VERSION_CODES.O)
		private fun mergeLogsApi26(sourceDir: String, outputFile: File) {
			val outputFilePath = Paths.get(outputFile.absolutePath)
			val logcatPath = Paths.get(sourceDir)

			Files.list(logcatPath)
				.sorted { o1, o2 ->
					Files.getLastModifiedTime(o1).compareTo(Files.getLastModifiedTime(o2))
				}
				.flatMap(Files::lines)
				.forEach { line ->
					Files.write(
						outputFilePath,
						(line + System.lineSeparator()).toByteArray(),
						StandardOpenOption.CREATE,
						StandardOpenOption.APPEND,
					)
				}
		}

		override suspend fun getLogFile(): Result<File> {
			stop()
			return withContext(Dispatchers.IO) {
				try {
					val outputDir = File(LogcatHelperInit.publicAppDirectory + File.separator + "output")
					val outputFile = File(outputDir.absolutePath + File.separator + "logs.txt")

					if (!outputDir.exists()) outputDir.mkdir()
					if (outputFile.exists()) outputFile.delete()

					if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
						mergeLogsApi26(LogcatHelperInit.logcatPath, outputFile)
					} else {
						mergeLogs(LogcatHelperInit.logcatPath, outputFile)
					}
					Result.success(outputFile)
				} catch (e: Exception) {
					Result.failure(e)
				} finally {
					start()
				}
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
		) : Thread() {
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

			override fun run() {
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
							deleteOldestFile(logcatPath)
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

			private fun deleteOldestFile(path: String) {
				val directory = File(path)
				if (directory.isDirectory) {
					directory.listFiles()?.toMutableList()?.run {
						this.sortBy { it.lastModified() }
						this.first().delete()
					}
				}
			}
		}
	}
}
