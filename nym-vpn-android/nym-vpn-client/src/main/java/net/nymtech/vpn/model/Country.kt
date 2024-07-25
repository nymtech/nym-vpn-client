package net.nymtech.vpn.model

import kotlinx.serialization.Serializable
import kotlinx.serialization.json.Json
import net.nymtech.vpn.util.Constants
import nym_vpn_lib.EntryPoint
import nym_vpn_lib.ExitPoint
import java.util.Locale

@Serializable
data class Country(
	val isoCode: String = Constants.DEFAULT_COUNTRY_ISO,
	val name: String = Locale(isoCode.lowercase(), isoCode).displayCountry,
	val isLowLatency: Boolean = false,
	val isDefault: Boolean = false,
) {
	init {
		if (isoCode.length > 2) {
			throw IllegalArgumentException("isoCode must be two characters")
		}
	}

	override fun toString(): String {
		return Json.encodeToString(serializer(), this)
	}

	fun toEntryPoint(): EntryPoint {
		return EntryPoint.Location(isoCode)
	}

	fun toExitPoint(): ExitPoint {
		return ExitPoint.Location(isoCode)
	}

	companion object {
		fun from(string: String?): Country? {
			return string?.let { Json.decodeFromString<Country>(string) }
		}

		fun fromCollectionString(string: String?): Set<Country> {
			return string?.let {
				Json.decodeFromString<Set<Country>>(it)
			} ?: emptySet()
		}
	}
}
