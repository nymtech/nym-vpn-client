package net.nymtech.nymvpn.ui.common

import net.nymtech.nymvpn.R
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNotEquals
import org.junit.Test

class FlagResourcesTest {

	@Test
	fun knownCountryResolvesToItsFlag() {
		assertEquals(R.drawable.flag_fr, FlagResources.forCountry("fr"))
		assertEquals(R.drawable.flag_fr, FlagResources.forCountry("FR"))
	}

	@Test
	fun unknownOrMissingCountryFallsBackToUnknownFlag() {
		assertEquals(R.drawable.flag_unknown, FlagResources.forCountry("zz"))
		assertEquals(R.drawable.flag_unknown, FlagResources.forCountry(null))
		assertNotEquals(R.drawable.flag_unknown, FlagResources.forCountry("us"))
	}
}
