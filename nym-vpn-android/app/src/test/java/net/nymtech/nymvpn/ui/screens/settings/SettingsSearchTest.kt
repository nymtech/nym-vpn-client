package net.nymtech.nymvpn.ui.screens.settings

import net.nymtech.nymvpn.ui.screens.settings.components.matchesSearch
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test

class SettingsSearchTest {

	@Test
	fun `matches part of a title ignoring case`() {
		assertTrue(matchesSearch("Customize DNS", "dns"))
	}

	@Test
	fun `ignores accents and surrounding spaces`() {
		assertTrue(matchesSearch("Écran de démarrage", "  ecran "))
	}

	@Test
	fun `does not match unrelated text`() {
		assertFalse(matchesSearch("Kill switch", "zzz"))
	}
}
