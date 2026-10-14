package net.nymtech.nymvpn.manager.backend

import org.junit.Assert.assertEquals
import org.junit.Test

class AppUpdateDecisionTest {
	@Test
	fun dismissible_belowFloor_showsAndAllowsConnect() {
		val decision = appUpdateDecision("2026.10.0", "2026.12.4", "dismissible")
		assertEquals(AppUpdateDecision(showDialog = true, blockConnect = false), decision)
	}

	@Test
	fun required_belowFloor_showsAndBlocksConnect() {
		val decision = appUpdateDecision("2026.10.0", "2026.12.4", "required")
		assertEquals(AppUpdateDecision(showDialog = true, blockConnect = true), decision)
	}

	@Test
	fun required_equalFloor_hidesAndAllowsConnect() {
		val decision = appUpdateDecision("2026.12.4", "2026.12.4", "required")
		assertEquals(AppUpdateDecision(showDialog = false, blockConnect = false), decision)
	}

	@Test
	fun missingCompatibility_hidesAndAllowsConnect() {
		val decision = appUpdateDecision("2026.13.0", null, "required")
		assertEquals(AppUpdateDecision(showDialog = false, blockConnect = false), decision)
	}

	@Test
	fun unknownPolicy_isDismissible() {
		val decision = appUpdateDecision("2026.10.0", "2026.12.4", "nope")
		assertEquals(AppUpdateDecision(showDialog = true, blockConnect = false), decision)
	}

	@Test
	fun platformOverride_beatsTheDefaultPolicy() {
		val required = appUpdateDecision("2026.10.0", "2026.12.4", "dismissible", mapOf("android" to "required"))
		assertEquals(AppUpdateDecision(showDialog = true, blockConnect = true), required)
		val dismissible = appUpdateDecision("2026.10.0", "2026.12.4", "required", mapOf("android" to "dismissible"))
		assertEquals(AppUpdateDecision(showDialog = true, blockConnect = false), dismissible)
		val otherPlatform = appUpdateDecision("2026.10.0", "2026.12.4", "dismissible", mapOf("ios" to "required"))
		assertEquals(AppUpdateDecision(showDialog = true, blockConnect = false), otherPlatform)
	}
}
