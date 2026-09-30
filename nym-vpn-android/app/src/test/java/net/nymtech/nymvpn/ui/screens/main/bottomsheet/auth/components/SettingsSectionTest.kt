package net.nymtech.nymvpn.ui.screens.main.bottomsheet.auth.components

import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test

class SettingsSectionTest {

	@Test
	fun errorReportsRowReadsAndWritesSentry() {
		assertTrue(techOptChecked(TechOptToggle.ErrorReports, statsEnabled = false, sentryEnabled = true))
		assertFalse(techOptChecked(TechOptToggle.ErrorReports, statsEnabled = true, sentryEnabled = false))

		var sentry: Boolean? = null
		var stats: Boolean? = null
		dispatchTechOptToggle(TechOptToggle.ErrorReports, true, { stats = it }, { sentry = it })
		assertEquals(true, sentry)
		assertEquals(null, stats)
	}

	@Test
	fun usageAnalyticsRowReadsAndWritesStatistics() {
		assertTrue(techOptChecked(TechOptToggle.UsageAnalytics, statsEnabled = true, sentryEnabled = false))
		assertFalse(techOptChecked(TechOptToggle.UsageAnalytics, statsEnabled = false, sentryEnabled = true))

		var sentry: Boolean? = null
		var stats: Boolean? = null
		dispatchTechOptToggle(TechOptToggle.UsageAnalytics, false, { stats = it }, { sentry = it })
		assertEquals(false, stats)
		assertEquals(null, sentry)
	}

	@Test
	fun onboardingPairStoresTheSameValuesSettingsShows() {
		var stats = true
		var sentry = false
		dispatchTechOptToggle(TechOptToggle.ErrorReports, true, { stats = it }, { sentry = it })
		dispatchTechOptToggle(TechOptToggle.UsageAnalytics, false, { stats = it }, { sentry = it })
		assertTrue(sentry)
		assertFalse(stats)
	}
}
