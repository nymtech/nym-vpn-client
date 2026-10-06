package net.nymtech.nymvpn.util.extensions

import android.content.ClipData
import android.content.ClipDescription
import android.content.ClipboardManager
import android.content.Context
import android.os.Build
import android.os.PersistableBundle
import android.widget.Toast
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.Job
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch
import net.nymtech.nymvpn.R

private const val EXTRA_IS_SENSITIVE_COMPAT = "android.content.extra.IS_SENSITIVE"
private const val SENSITIVE_CLIP_CLEAR_DELAY_SECONDS = 60

private val clipboardClearScope = CoroutineScope(SupervisorJob() + Dispatchers.Main)
private var clipboardClearJob: Job? = null

private fun sensitiveClipData(text: String): ClipData = ClipData.newPlainText("", text).apply {
	val key = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) ClipDescription.EXTRA_IS_SENSITIVE else EXTRA_IS_SENSITIVE_COMPAT
	description.extras = PersistableBundle().apply { putBoolean(key, true) }
}

/** Copies [text] as sensitive and tries to clear it after [SENSITIVE_CLIP_CLEAR_DELAY_SECONDS] (best-effort) */
fun Context.copySensitiveToClipboard(text: String) {
	val clipboard = applicationContext.getSystemService(ClipboardManager::class.java) ?: return
	val clip = sensitiveClipData(text)
	clipboard.setPrimaryClip(clip)
	Toast.makeText(this, getString(R.string.clipboard_sensitive_copied, SENSITIVE_CLIP_CLEAR_DELAY_SECONDS), Toast.LENGTH_LONG).show()
	clipboardClearJob?.cancel()
	clipboardClearJob = clipboardClearScope.launch {
		delay(SENSITIVE_CLIP_CLEAR_DELAY_SECONDS * 1_000L)
		if (!clipboard.isCurrentClip(clip)) return@launch
		if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.P) {
			clipboard.clearPrimaryClip()
		} else {
			clipboard.setPrimaryClip(ClipData.newPlainText("", ""))
		}
	}
}

private fun ClipboardManager.isCurrentClip(clip: ClipData): Boolean {
	val current = primaryClip ?: return true
	return current.itemCount == 1 && current.getItemAt(0).text?.toString() == clip.getItemAt(0).text?.toString()
}
