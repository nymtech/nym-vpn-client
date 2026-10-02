package net.nymtech.nymvpn.util.extensions

import android.content.ClipData
import android.content.ClipDescription
import android.os.Build
import android.os.PersistableBundle

private const val EXTRA_IS_SENSITIVE_COMPAT = "android.content.extra.IS_SENSITIVE"

fun sensitiveClipData(text: String): ClipData = ClipData.newPlainText("", text).apply {
	val key = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) ClipDescription.EXTRA_IS_SENSITIVE else EXTRA_IS_SENSITIVE_COMPAT
	description.extras = PersistableBundle().apply { putBoolean(key, true) }
}
