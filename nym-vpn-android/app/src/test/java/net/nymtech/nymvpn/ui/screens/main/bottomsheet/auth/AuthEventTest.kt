package net.nymtech.nymvpn.ui.screens.main.bottomsheet.auth

import org.junit.Assert.assertEquals
import org.junit.Test

class AuthEventTest {

	@Test
	fun loginMnemonicImported_isSinglePostStoreEvent() {
		val phrase = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about"
		val event = AuthEvent.LoginMnemonicImported(phrase)
		assertEquals(phrase, event.phrase)
	}
}
