package net.nymtech.nymvpn.ui.screens.settings.privacy.components

import androidx.annotation.StringRes
import net.nymtech.nymvpn.R

/**
 * Android keeps two logging switches under the hood: whether logs are kept at all
 * (settings) and whether the core logs at DEBUG (core config). Debug only makes sense
 * with logging on, so users get one choice with three levels instead of two switches.
 */
enum class LoggingLevel(@StringRes val labelRes: Int, @StringRes val descriptionRes: Int) {
	OFF(R.string.logs_level_off, R.string.logs_level_off_description),
	BASIC(R.string.logs_level_basic, R.string.logs_level_basic_description),
	DEBUG(R.string.logs_level_debug, R.string.logs_level_debug_description),
	;

	companion object {
		fun from(logsEnabled: Boolean, debugEnabled: Boolean): LoggingLevel = when {
			!logsEnabled -> OFF
			debugEnabled -> DEBUG
			else -> BASIC
		}
	}
}
