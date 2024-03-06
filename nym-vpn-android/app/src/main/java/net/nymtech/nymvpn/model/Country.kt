package net.nymtech.nymvpn.model

import kotlinx.serialization.Serializable
import kotlinx.serialization.json.Json
import net.nymtech.nymvpn.util.Constants
import java.util.Locale

typealias Countries = List<Country>
@Serializable
data class Country(
    val isoCode: String = Constants.DEFAULT_COUNTRY_ISO,
    val name: String = Locale(isoCode.lowercase(), isoCode).displayCountry,
    val isFastest: Boolean = false
) {
    override fun toString(): String {
        return Json.encodeToString(serializer(), this)
    }

    companion object {
        //TODO handle errors
        fun from(string: String): Country {
            return Json.decodeFromString<Country>(string)
        }
        fun fromCollectionString(string: String) : Countries {
            return Json.decodeFromString<Countries>(string)
        }
    }
}