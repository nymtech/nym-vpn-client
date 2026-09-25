package net.nymtech.nymvpn.manager.backend

import kotlinx.coroutines.runBlocking
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.screens.main.modal.compatibilityCopy
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
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
	fun prerelease_isOlderThanTheRelease() {
		val decision = appUpdateDecision("2026.13.0-beta.1", "2026.13.0", "dismissible")
		assertEquals(AppUpdateDecision(showDialog = true, blockConnect = false), decision)
	}

	@Test
	fun unknownPolicy_isDismissible() {
		val decision = appUpdateDecision("2026.10.0", "2026.12.4", "nope")
		assertEquals(AppUpdateDecision(showDialog = true, blockConnect = false), decision)
	}

	@Test
	fun emptyPolicy_isDismissible() {
		val decision = appUpdateDecision("2026.10.0", "2026.12.4", "")
		assertEquals(AppUpdateDecision(showDialog = true, blockConnect = false), decision)
	}

	@Test
	fun unparseableFloor_hidesAndAllowsConnect() {
		val decision = appUpdateDecision("2026.13.0", "not-a-version", "required")
		assertEquals(AppUpdateDecision(showDialog = false, blockConnect = false), decision)
	}

	@Test
	fun unresolvedGateWaitsForTheInitializedDecision() = runBlocking {
		val gate = resolvedUpdateGate(UpdateGate(initialized = false, blockConnect = false)) {
			UpdateGate(initialized = true, blockConnect = true)
		}
		assertEquals(UpdateGate(initialized = true, blockConnect = true), gate)
	}

	@Test
	fun initializedRequiredGateDoesNotWait() = runBlocking {
		val gate = resolvedUpdateGate(UpdateGate(initialized = true, blockConnect = true)) {
			error("startTunnel must not read the decision before init")
		}
		assertTrue(gate.blockConnect)
	}

	@Test
	fun dismissibleCopyIsNotTheUnsupportedCopy() {
		val dismissible = compatibilityCopy(allowDismiss = true)
		val required = compatibilityCopy(allowDismiss = false)
		assertEquals(R.string.app_update_available_title, dismissible.first)
		assertEquals(R.string.app_update_available_body, dismissible.second)
		assertEquals(R.string.update_required, required.first)
		assertEquals(R.string.app_update_required, required.second)
	}
}
