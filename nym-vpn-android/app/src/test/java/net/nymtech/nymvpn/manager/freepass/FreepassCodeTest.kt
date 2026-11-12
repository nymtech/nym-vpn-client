package net.nymtech.nymvpn.manager.freepass

import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Test

class FreepassCodeTest {

	@Test
	fun acceptsBareCodesAndRedemptionLinks() {
		mapOf(
			"NYM2026" to "NYM2026",
			"  abcd_1234 \n" to "abcd_1234",
			"https://nym.com/account/freepass?code=NYM2026" to "NYM2026",
			"https://www.nym.com/account/freepass?code=NYM2026" to "NYM2026",
			"https://nym.com/account/freepass?utm_source=qr&code=NYM2026" to "NYM2026",
			"nym.com/account/freepass?code=gsjK6Yb1E3e" to "gsjK6Yb1E3e",
			"https://nym.com/download#code=gsjK6Yb1E3e" to "gsjK6Yb1E3e",
			"nym.com/download#code=gsjK6Yb1E3e" to "gsjK6Yb1E3e",
			"www.nym.com/download?utm_source=qr#code=gsjK6Yb1E3e" to "gsjK6Yb1E3e",
		).forEach { (contents, expected) -> assertEquals(contents, expected, FreepassCode.parse(contents)) }
	}

	@Test
	fun rejectsAnythingElse() {
		listOf(
			"",
			"abc",
			"a".repeat(31),
			"NYM-2026",
			"http://nym.com/account/freepass?code=NYM2026",
			"https://nym.com.evil.example/account/freepass?code=NYM2026",
			"https://user@nym.com/account/freepass?code=NYM2026",
			"https://nym.com:8443/account/freepass?code=NYM2026",
			"https://nym.com/account/other?code=NYM2026",
			"https://nym.com/account/freepass?code=NYM2026#frag",
			"https://nym.com/account/freepass?code=NYM%2D2026",
			"https://nym.com/account/freepass",
			"https://nym.com/download?code=NYM2026",
			"https://nym.com/download#code=NYM-2026",
			"http://nym.com/download#code=NYM2026",
			"evil.example/download#code=NYM2026",
			"nym.com.evil.example/download#code=NYM2026",
			"https://nym.com/qr-pairing?pubkey=9Ez7Qw3hLk2sVtYpRbN4cJmA6uXfD8gH1iKoP5rStUvW",
		).forEach { assertNull(it, FreepassCode.parse(it)) }
	}
}
