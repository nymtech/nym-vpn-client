package net.nymtech.vpn.util

import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Test

private const val INVALID_UID = -1

class ConnectionOwnerResolverTest {
	@Test
	fun `parses ipv4 addr port`() {
		val parsed = ConnectionOwnerResolver.parseAddrPort("10.0.0.2:443")!!
		assertEquals("10.0.0.2", parsed.hostString)
		assertEquals(443, parsed.port)
	}

	@Test
	fun `parses bracketed ipv6 addr port`() {
		val parsed = ConnectionOwnerResolver.parseAddrPort("[fd00::1]:53")!!
		assertEquals("fd00::1", parsed.hostString)
		assertEquals(53, parsed.port)
	}

	@Test
	fun `rejects garbage`() {
		assertNull(ConnectionOwnerResolver.parseAddrPort("not-an-address"))
		assertNull(ConnectionOwnerResolver.parseAddrPort("10.0.0.2"))
		assertNull(ConnectionOwnerResolver.parseAddrPort("[fd00::1]:notaport"))
	}

	@Test
	fun `lookup fails closed when ConnectivityManager is null`() {
		assertEquals(INVALID_UID, ConnectionOwnerResolver.lookup(null, 6, "10.0.0.2:443", "10.0.0.3:80"))
	}
}
