package net.nymtech.nymvpn.ui.common.security

import android.os.Build
import android.view.Window
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.ui.platform.LocalView
import androidx.compose.ui.window.DialogWindowProvider
import net.nymtech.nymvpn.util.extensions.findActivity
import java.util.WeakHashMap

/** Hides other apps' overlays (API 31+) or filters obscured touches while in composition.
 * Scoped to sensitive screens, not app-wide, so legit overlays (dimmers, bubbles) keep working elsewhere. */
@Composable
fun SecureOverlayProtection() {
	val view = LocalView.current
	if (view.isInEditMode) return
	DisposableEffect(view) {
		val windows = listOfNotNull(
			(view.parent as? DialogWindowProvider)?.window,
			view.context.findActivity()?.window,
		).distinct()
		windows.forEach(OverlayProtection::acquire)
		onDispose {
			windows.forEach(OverlayProtection::release)
		}
	}
}

private object OverlayProtection {
	private class Entry(var count: Int, val previousFilterTouches: Boolean)

	private val entries = WeakHashMap<Window, Entry>()

	fun acquire(window: Window) {
		entries[window]?.let {
			it.count++
			return
		}
		val decorView = window.decorView
		entries[window] = Entry(count = 1, previousFilterTouches = decorView.filterTouchesWhenObscured)
		if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.S) {
			window.setHideOverlayWindows(true)
		} else {
			decorView.filterTouchesWhenObscured = true
		}
	}

	fun release(window: Window) {
		val entry = entries[window] ?: return
		if (--entry.count > 0) return
		entries.remove(window)
		if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.S) {
			window.setHideOverlayWindows(false)
		} else {
			window.decorView.filterTouchesWhenObscured = entry.previousFilterTouches
		}
	}
}
