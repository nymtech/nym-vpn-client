package net.nymtech.nymvpn.ui.screens.settings.logs

import android.content.ContentValues
import android.content.Context
import android.net.Uri
import android.os.Build
import android.provider.MediaStore
import androidx.core.content.FileProvider
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.Job
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.collectLatest
import kotlinx.coroutines.flow.receiveAsFlow
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import net.nymtech.logcatutil.LogReader
import net.nymtech.logcatutil.model.LogMessage
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.data.SettingsRepository
import net.nymtech.nymvpn.data.config.VpnConfigRepository
import net.nymtech.nymvpn.di.qualifiers.IoDispatcher
import net.nymtech.nymvpn.di.qualifiers.MainDispatcher
import net.nymtech.nymvpn.ui.common.snackbar.SnackbarController
import net.nymtech.nymvpn.util.Constants
import net.nymtech.nymvpn.util.StringValue
import net.nymtech.nymvpn.util.extensions.chunked
import net.nymtech.nymvpn.util.extensions.launchShareFile
import net.nymtech.vpn.config.CoreVpnConfigUpdate
import timber.log.Timber
import java.io.File
import java.time.Duration
import java.time.Instant
import javax.inject.Inject

@HiltViewModel
class LogsViewModel @Inject constructor(
	private val logReader: LogReader,
	private val settingsRepository: SettingsRepository,
	private val vpnConfigRepository: VpnConfigRepository,
	@IoDispatcher private val ioDispatcher: CoroutineDispatcher,
	@MainDispatcher private val mainDispatcher: CoroutineDispatcher,
) : ViewModel() {

	companion object {
		private const val TAG = "ui-logs-vm"
	}

	private val _appLogs = MutableStateFlow<List<LogMessage>>(emptyList())
	val appLogs: StateFlow<List<LogMessage>> = _appLogs.asStateFlow()

	private val _tunnelLogs = MutableStateFlow<List<LogMessage>>(emptyList())
	val tunnelLogs: StateFlow<List<LogMessage>> = _tunnelLogs.asStateFlow()

	private val _libraryLogs = MutableStateFlow<List<LogMessage>>(emptyList())
	val libraryLogs: StateFlow<List<LogMessage>> = _libraryLogs.asStateFlow()

	private val _requestSaveUri = Channel<String>(Channel.BUFFERED)
	val requestSaveUri = _requestSaveUri.receiveAsFlow()

	init {
		viewModelScope.launch(ioDispatcher) {
			logReader.bufferedLogsApp
				.chunked(200, Duration.ofMillis(500))
				.collectLatest { logsChunk ->
					withContext(mainDispatcher) {
						_appLogs.value = (_appLogs.value + logsChunk)
							.takeLast(Constants.LOG_BUFFER_SIZE.toInt())
					}
				}
		}

		viewModelScope.launch(ioDispatcher) {
			logReader.bufferedLogsTunnel
				.chunked(200, Duration.ofMillis(500))
				.collectLatest { logsChunk ->
					withContext(mainDispatcher) {
						_tunnelLogs.value = (_tunnelLogs.value + logsChunk)
							.takeLast(Constants.LOG_BUFFER_SIZE.toInt())
					}
				}
		}

		viewModelScope.launch(ioDispatcher) {
			logReader.bufferedLogsLibrary
				.chunked(200, Duration.ofMillis(500))
				.collectLatest { logsChunk ->
					withContext(mainDispatcher) {
						_libraryLogs.value = (_libraryLogs.value + logsChunk)
							.takeLast(Constants.LOG_BUFFER_SIZE.toInt())
					}
				}
		}
	}

	fun shareLogs(context: Context): Job = viewModelScope.launch(ioDispatcher) {
		Timber.tag(TAG).i("LogsShareRequested")

		runCatching {
			val sharePath = File(context.filesDir, "external_files")
			if (sharePath.exists()) sharePath.delete()
			sharePath.mkdir()

			val file = File("${sharePath.path}/${Constants.BASE_LOG_FILE_NAME}-${Instant.now().epochSecond}.zip")
			if (file.exists()) file.delete()
			file.createNewFile()

			logReader.zipLogFiles(file.absolutePath)

			val uri = FileProvider.getUriForFile(context, context.getString(R.string.provider), file)
			context.launchShareFile(uri)

			Timber.tag(TAG).i("LogsShareSuccess")
		}.onFailure { t ->
			Timber.tag(TAG).e(t, "LogsShareFailed")
		}
	}

	fun downloadLogs(context: Context): Job = viewModelScope.launch(ioDispatcher) {
		Timber.tag(TAG).i("LogsDownloadRequested")

		runCatching {
			val fileName = "${Constants.BASE_LOG_FILE_NAME}-${Instant.now().epochSecond}.zip"

			if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q) {
				val contentValues = ContentValues().apply {
					put(MediaStore.MediaColumns.DISPLAY_NAME, fileName)
					put(MediaStore.MediaColumns.MIME_TYPE, "application/zip")
					put(MediaStore.MediaColumns.RELATIVE_PATH, "Download/")
				}

				val resolver = context.contentResolver
				val uri = resolver.insert(MediaStore.Downloads.EXTERNAL_CONTENT_URI, contentValues)
					?: throw IllegalStateException("Failed to create MediaStore record")

				val tempFile = File(context.cacheDir, fileName)
				if (tempFile.exists()) tempFile.delete()
				tempFile.createNewFile()

				logReader.downloadFile(resolver, uri, tempFile)
				tempFile.delete()

				withContext(mainDispatcher) {
					SnackbarController.showMessage(StringValue.StringResource(R.string.logs_saved))
				}

				Timber.tag(TAG).i("LogsDownloadSuccess api=Q_plus")
			} else {
				_requestSaveUri.send(fileName)
				Timber.tag(TAG).i("LogsDownloadRequestedLegacy flow=ACTION_CREATE_DOCUMENT")
			}
		}.onFailure { t ->
			Timber.tag(TAG).e(t, "LogsDownloadFailed")
		}
	}

	fun deleteLogs() = viewModelScope.launch {
		Timber.tag(TAG).i("LogsDeleteRequested")

		runCatching {
			logReader.deleteAndClearLogs()
			_appLogs.value = emptyList()
			_tunnelLogs.value = emptyList()
			_libraryLogs.value = emptyList()
			Timber.tag(TAG).i("LogsDeleteSuccess")
		}.onFailure { t ->
			Timber.tag(TAG).e(t, "LogsDeleteFailed")
		}
	}

	fun saveLogsToUri(context: Context, uri: Uri) = viewModelScope.launch(ioDispatcher) {
		Timber.tag(TAG).i("LogsSaveToUriRequested")

		runCatching {
			val tempFile = File(
				context.cacheDir,
				"${Constants.BASE_LOG_FILE_NAME}-${Instant.now().epochSecond}.zip",
			)
			if (tempFile.exists()) tempFile.delete()
			tempFile.createNewFile()

			val resolver = context.contentResolver
			logReader.downloadFile(resolver, uri, tempFile)
			tempFile.delete()

			withContext(mainDispatcher) {
				SnackbarController.showMessage(StringValue.StringResource(R.string.logs_saved))
			}

			Timber.tag(TAG).i("LogsSaveToUriSuccess")
		}.onFailure { t ->
			Timber.tag(TAG).e(t, "LogsSaveToUriFailed")
		}
	}

	fun onLogsEnabled(enabled: Boolean) = viewModelScope.launch {
		Timber.tag(TAG).i("LogsEnabledChanged enabled=%s", enabled)
		settingsRepository.setLogsEnabled(enabled)
		if (!enabled) {
			onLogsDebugEnabled(false)
		}
	}

	fun onLogsDebugEnabled(enabled: Boolean) = viewModelScope.launch {
		Timber.tag(TAG).i("LogsDebugEnabledChanged enabled=%s", enabled)
		vpnConfigRepository.apply(CoreVpnConfigUpdate.SetDebugLog(enabled))
	}
}
