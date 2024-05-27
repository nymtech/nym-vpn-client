package net.nymtech.nymvpn.util

import android.content.ContentValues
import android.content.Context
import android.os.Build
import android.os.Environment
import android.provider.MediaStore
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.withContext
import java.io.File
import java.io.FileInputStream
import java.io.FileOutputStream

class FileUtils(
	private val context: Context,
	private val ioDispatcher: CoroutineDispatcher,
) {

	suspend fun readBytesFromFile(file: File): ByteArray {
		return withContext(ioDispatcher) {
			FileInputStream(file).use {
				it.readBytes()
			}
		}
	}

	suspend fun readTextFromFileName(fileName: String): String {
		return withContext(ioDispatcher) {
			context.assets.open(fileName).use { stream ->
				stream.bufferedReader(Charsets.UTF_8).use {
					it.readText()
				}
			}
		}
	}

	suspend fun saveByteArrayToDownloads(content: ByteArray, fileName: String): Result<Unit> {
		return withContext(ioDispatcher) {
			try {
				if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q) {
					val contentValues =
						ContentValues().apply {
							put(MediaStore.MediaColumns.DISPLAY_NAME, fileName)
							put(MediaStore.MediaColumns.MIME_TYPE, Constants.TEXT_MIME_TYPE)
							put(MediaStore.MediaColumns.RELATIVE_PATH, Environment.DIRECTORY_DOWNLOADS)
						}
					val resolver = context.contentResolver
					val uri = resolver.insert(MediaStore.Downloads.EXTERNAL_CONTENT_URI, contentValues)
					if (uri != null) {
						resolver.openOutputStream(uri).use { output ->
							output?.write(content)
						}
					}
				} else {
					val target =
						File(
							Environment.getExternalStoragePublicDirectory(Environment.DIRECTORY_DOWNLOADS),
							fileName,
						)
					FileOutputStream(target).use { output ->
						output.write(content)
					}
				}
				Result.success(Unit)
			} catch (e: Exception) {
				Result.failure(e)
			}
		}
	}
}
