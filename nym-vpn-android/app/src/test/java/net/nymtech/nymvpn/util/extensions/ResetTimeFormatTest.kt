package net.nymtech.nymvpn.util.extensions

import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Test
import java.time.ZoneId

class ResetTimeFormatTest {

	// 2026-10-01T00:00:00Z as epoch seconds
	private val midnightUtc = 1_790_812_800L

	@Test
	fun formatResetTime_rendersInGivenLocalZone() {
		assertEquals("02:00", formatResetTime(midnightUtc, ZoneId.of("Europe/Zurich")))
		assertEquals("20:00", formatResetTime(midnightUtc, ZoneId.of("America/New_York")))
	}

	@Test
	fun formatResetTime_acceptsEpochMillis() {
		assertEquals("02:00", formatResetTime(midnightUtc * 1000, ZoneId.of("Europe/Zurich")))
	}

	@Test
	fun formatResetTime_hasNoUtcSuffix() {
		assertFalse(formatResetTime(midnightUtc, ZoneId.of("UTC")).contains("UTC"))
	}

	@Test
	fun formatResetTime_nullShowsPlaceholder() {
		assertEquals("—", formatResetTime(null, ZoneId.of("Europe/Zurich")))
	}
}
