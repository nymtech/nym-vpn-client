package net.nymtech.nymvpn.util

import net.nymtech.nymvpn.ui.screens.settings.tunneling.AppInfo
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test

class SplitTunnelingAllAppsTest {

	private fun app(packageName: String, passThroughVpn: Boolean) = AppInfo(
		name = packageName,
		packageName = packageName,
		icon = 0,
		passThroughVpn = passThroughVpn,
	)

	@Test
	fun setAllPassThroughValue_true_routesEveryAppViaVpn() {
		val apps = listOf(app("a", false), app("b", true), app("c", false))

		val result = apps.setAllPassThroughValue(true)

		assertEquals(listOf(true, true, true), result.map { it.passThroughVpn })
		assertEquals(listOf("a", "b", "c"), result.map { it.packageName })
	}

	@Test
	fun setAllPassThroughValue_false_routesEveryAppDirect() {
		val apps = listOf(app("a", false), app("b", true), app("c", true))

		val result = apps.setAllPassThroughValue(false)

		assertEquals(listOf(false, false, false), result.map { it.passThroughVpn })
	}

	@Test
	fun allPassThroughVpn_isTrueOnlyWhenEveryAppIsViaVpn() {
		assertTrue(listOf(app("a", true), app("b", true)).allPassThroughVpn())
		assertFalse(listOf(app("a", true), app("b", false)).allPassThroughVpn())
		assertFalse(listOf(app("a", false), app("b", false)).allPassThroughVpn())
	}

	@Test
	fun allPassThroughVpn_isFalseForEmptyList() {
		assertFalse(emptyList<AppInfo>().allPassThroughVpn())
	}
}
