package net.nymtech.nymvpn.manager.pairing

import kotlinx.serialization.Serializable
import kotlinx.serialization.json.Json
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test

class PairingQrCodeTest {

	@Serializable
	private data class Vectors(val description: String, val vectors: List<Vector>)

	@Serializable
	private data class Vector(val name: String, val contents: String, val publicKey: String?)

	private val vectors: List<Vector> by lazy {
		val text = checkNotNull(javaClass.getResource("/pairing/qr-pairing-vectors.json")).readText()
		Json.decodeFromString<Vectors>(text).vectors
	}

	@Test
	fun matchesSharedVectors() {
		assertTrue(vectors.isNotEmpty())
		vectors.forEach { vector ->
			assertEquals(vector.name, vector.publicKey, PairingQrCode.parsePublicKey(vector.contents))
		}
	}

	@Test
	fun rejectsOversizedText() {
		val contents = "https://nym.com/qr-pairing?pubkey=" + "1".repeat(PairingQrCode.MAX_LENGTH)
		assertNull(PairingQrCode.parsePublicKey(contents))
	}
}
