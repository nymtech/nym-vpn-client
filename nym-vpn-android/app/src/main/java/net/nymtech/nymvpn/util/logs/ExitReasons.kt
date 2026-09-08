package net.nymtech.nymvpn.util.logs

import android.app.ApplicationExitInfo
import java.time.Instant

/** Formats ApplicationExitInfo records into single log lines. */
object ExitReasons {

	fun reasonName(reason: Int): String = when (reason) {
		ApplicationExitInfo.REASON_UNKNOWN -> "UNKNOWN"
		ApplicationExitInfo.REASON_EXIT_SELF -> "EXIT_SELF"
		ApplicationExitInfo.REASON_SIGNALED -> "SIGNALED"
		ApplicationExitInfo.REASON_LOW_MEMORY -> "LOW_MEMORY"
		ApplicationExitInfo.REASON_CRASH -> "CRASH"
		ApplicationExitInfo.REASON_CRASH_NATIVE -> "CRASH_NATIVE"
		ApplicationExitInfo.REASON_ANR -> "ANR"
		ApplicationExitInfo.REASON_INITIALIZATION_FAILURE -> "INITIALIZATION_FAILURE"
		ApplicationExitInfo.REASON_PERMISSION_CHANGE -> "PERMISSION_CHANGE"
		ApplicationExitInfo.REASON_EXCESSIVE_RESOURCE_USAGE -> "EXCESSIVE_RESOURCE_USAGE"
		ApplicationExitInfo.REASON_USER_REQUESTED -> "USER_REQUESTED"
		ApplicationExitInfo.REASON_USER_STOPPED -> "USER_STOPPED"
		ApplicationExitInfo.REASON_DEPENDENCY_DIED -> "DEPENDENCY_DIED"
		ApplicationExitInfo.REASON_OTHER -> "OTHER"
		ApplicationExitInfo.REASON_FREEZER -> "FREEZER"
		ApplicationExitInfo.REASON_PACKAGE_STATE_CHANGE -> "PACKAGE_STATE_CHANGE"
		ApplicationExitInfo.REASON_PACKAGE_UPDATED -> "PACKAGE_UPDATED"
		else -> "UNKNOWN($reason)"
	}

	fun formatLine(timestampMs: Long, reason: Int, status: Int, importance: Int, description: String?): String {
		val base = "PriorExit time=${Instant.ofEpochMilli(timestampMs)} reason=${reasonName(reason)} " +
			"status=$status importance=$importance"
		val singleLineDescription = description?.replace(Regex("\\s*[\r\n]+\\s*"), " ")?.trim()
		return if (singleLineDescription.isNullOrBlank()) base else "$base description=$singleLineDescription"
	}
}
