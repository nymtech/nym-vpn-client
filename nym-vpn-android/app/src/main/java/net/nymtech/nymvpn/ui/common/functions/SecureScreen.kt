package net.nymtech.nymvpn.ui.common.functions

import android.view.WindowManager
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.ui.platform.LocalView
import androidx.compose.ui.window.DialogWindowProvider
import net.nymtech.nymvpn.util.extensions.findActivity

/** Blocks screenshots and screen recording while in composition. */

@Composable
fun SecureScreen() {
	val view = LocalView.current
	if (view.isInEditMode) return
	DisposableEffect(view) {
		val windows = listOfNotNull(
			(view.parent as? DialogWindowProvider)?.window,
			view.context.findActivity()?.window,
		).distinct()
		windows.forEach { it.addFlags(WindowManager.LayoutParams.FLAG_SECURE) }
		onDispose {
			windows.forEach { it.clearFlags(WindowManager.LayoutParams.FLAG_SECURE) }
		}
	}
}
