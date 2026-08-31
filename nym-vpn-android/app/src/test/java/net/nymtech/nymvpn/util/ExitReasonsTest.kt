package net.nymtech.nymvpn.util

import android.app.ApplicationExitInfo
import net.nymtech.nymvpn.util.logs.ExitReasons
import org.junit.Assert.assertEquals
import org.junit.Test

class ExitReasonsTest {

	@Test
	fun reasonName_mapsKnownCodes() {
		assertEquals("CRASH_NATIVE", ExitReasons.reasonName(ApplicationExitInfo.REASON_CRASH_NATIVE))
		assertEquals("LOW_MEMORY", ExitReasons.reasonName(ApplicationExitInfo.REASON_LOW_MEMORY))
		assertEquals("USER_REQUESTED", ExitReasons.reasonName(ApplicationExitInfo.REASON_USER_REQUESTED))
		assertEquals("ANR", ExitReasons.reasonName(ApplicationExitInfo.REASON_ANR))
		assertEquals("SIGNALED", ExitReasons.reasonName(ApplicationExitInfo.REASON_SIGNALED))
	}

	@Test
	fun reasonName_fallsBackToCodeForUnknown() {
		assertEquals("UNKNOWN(99)", ExitReasons.reasonName(99))
	}

	@Test
	fun formatLine_includesTimestampReasonStatusImportanceAndDescription() {
		val line = ExitReasons.formatLine(
			timestampMs = 1787849947000,
			reason = ApplicationExitInfo.REASON_USER_REQUESTED,
			status = 0,
			importance = 400,
			description = "stop net.nymtech.nymvpn due to from pid 1234",
		)
		assertEquals(
			"PriorExit time=2026-08-27T16:59:07Z reason=USER_REQUESTED status=0 importance=400 description=stop net.nymtech.nymvpn due to from pid 1234",
			line,
		)
	}

	@Test
	fun formatLine_omitsDescriptionWhenNull() {
		val line = ExitReasons.formatLine(
			timestampMs = 1787849947000,
			reason = ApplicationExitInfo.REASON_LOW_MEMORY,
			status = 0,
			importance = 300,
			description = null,
		)
		assertEquals(
			"PriorExit time=2026-08-27T16:59:07Z reason=LOW_MEMORY status=0 importance=300",
			line,
		)
	}

	@Test
	fun formatLine_collapsesMultiLineDescriptionToSingleLine() {
		val line = ExitReasons.formatLine(
			timestampMs = 1787849947000,
			reason = ApplicationExitInfo.REASON_CRASH_NATIVE,
			status = 0,
			importance = 100,
			description = "signal 11 (SIGSEGV)\r\nbacktrace:\n  #00 pc 0001",
		)
		assertEquals(
			"PriorExit time=2026-08-27T16:59:07Z reason=CRASH_NATIVE status=0 importance=100 " +
				"description=signal 11 (SIGSEGV) backtrace: #00 pc 0001",
			line,
		)
	}
}
