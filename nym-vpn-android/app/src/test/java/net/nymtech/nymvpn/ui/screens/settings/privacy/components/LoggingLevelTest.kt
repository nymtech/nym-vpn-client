package net.nymtech.nymvpn.ui.screens.settings.privacy.components

import org.junit.Assert.assertEquals
import org.junit.Test

class LoggingLevelTest {

	@Test
	fun from_readsEveryStoredCombination() {
		assertEquals(LoggingLevel.OFF, LoggingLevel.from(logsEnabled = false, debugEnabled = false))
		assertEquals(LoggingLevel.OFF, LoggingLevel.from(logsEnabled = false, debugEnabled = true))
		assertEquals(LoggingLevel.BASIC, LoggingLevel.from(logsEnabled = true, debugEnabled = false))
		assertEquals(LoggingLevel.DEBUG, LoggingLevel.from(logsEnabled = true, debugEnabled = true))
	}

	@Test
	fun from_readsBackWhatEachLevelWrites() {
		// What PrivacyViewModel.onLoggingLevelChange stores for each level once both writes land.
		LoggingLevel.entries.forEach { level ->
			val logsEnabled = level != LoggingLevel.OFF
			val debugEnabled = level == LoggingLevel.DEBUG
			assertEquals(level, LoggingLevel.from(logsEnabled, debugEnabled))
		}
	}
}
