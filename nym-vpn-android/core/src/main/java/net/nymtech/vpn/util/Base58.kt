package net.nymtech.vpn.util

import timber.log.Timber
import java.math.BigInteger

object Base58 {
	private const val ALPHABET = "123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz"
	private val INDEXES = IntArray(128) { -1 }.also {
		for (i in ALPHABET.indices) {
			it[ALPHABET[i].code] = i
		}
	}

	fun isValidBase58(input: String, expectedByteLength: Int = 32): Boolean {
		try {
			if (input.isEmpty() || input.any { it.code >= 128 || INDEXES[it.code] == -1 }) {
				return false
			}

			val bytes = decode(input)

			return bytes.size == expectedByteLength
		} catch (e: IllegalArgumentException) {
			Timber.e(e)
			return false
		}
	}

	private fun decode(input: String): ByteArray {
		if (input.isEmpty()) return ByteArray(0)

		val bigInt = input.fold(BigInteger.ZERO) { acc, char ->
			val index = INDEXES[char.code]
			if (index == -1) throw IllegalArgumentException("Invalid Base58 character: $char")
			acc.multiply(BigInteger.valueOf(58)).add(BigInteger.valueOf(index.toLong()))
		}

		val bytes = bigInt.toByteArray()

		val leadingZeros = input.takeWhile { it == '1' }.length
		return if (bytes[0].toInt() == 0 && bytes.size > 1) {
			ByteArray(leadingZeros) + bytes.drop(1).toByteArray()
		} else {
			ByteArray(leadingZeros) + bytes
		}
	}
}
