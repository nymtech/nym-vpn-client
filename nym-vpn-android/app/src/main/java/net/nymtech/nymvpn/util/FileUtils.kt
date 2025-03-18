package net.nymtech.nymvpn.util

import android.content.Context
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.withContext

class FileUtils(
	private val context: Context,
	private val ioDispatcher: CoroutineDispatcher,
) {
	suspend fun readTextFromAssetsFile(fileName: String): Result<String> {
		return kotlin.runCatching {
			withContext(ioDispatcher) {
				context.assets.open(fileName).use { stream ->
					stream.bufferedReader(Charsets.UTF_8).use {
						it.readText()
					}
				}
			}
		}
	}
}
