package net.nymtech.nymvpn.ui.screens.settings.tunneling.components

import android.graphics.Bitmap
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.platform.LocalContext
import androidx.core.graphics.drawable.toBitmapOrNull

@Composable
fun LoadIcon(packageName: String): Bitmap? {
	val context = LocalContext.current
	val packageManager = remember(context) { context.packageManager }
	return try {
		packageManager.getApplicationIcon(packageName).toBitmapOrNull()
	} catch (_: Exception) {
		null
	}
}
