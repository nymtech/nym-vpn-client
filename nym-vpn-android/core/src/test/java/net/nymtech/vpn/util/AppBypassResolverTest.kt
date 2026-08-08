package net.nymtech.vpn.util

import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test

class AppBypassResolverTest {
	private val apps = listOf("com.example.app")

	@Test
	fun `steers when lockdown and exclusions on Q+`() {
		assertTrue(AppBypassResolver.shouldSteer(sdkInt = 29, lockdownEnabled = true, restrictedApps = apps))
	}

	@Test
	fun `no steering without lockdown`() {
		assertFalse(AppBypassResolver.shouldSteer(sdkInt = 34, lockdownEnabled = false, restrictedApps = apps))
	}

	@Test
	fun `no steering with empty exclusion list`() {
		assertFalse(AppBypassResolver.shouldSteer(sdkInt = 34, lockdownEnabled = true, restrictedApps = emptyList()))
	}

	@Test
	fun `no steering below api 29`() {
		assertFalse(AppBypassResolver.shouldSteer(sdkInt = 28, lockdownEnabled = true, restrictedApps = apps))
	}
}
