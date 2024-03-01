package net.nymtech.nymvpn.model

import kotlinx.serialization.Serializable
import kotlinx.serialization.json.Json

typealias Countries = List<Country>
@Serializable
data class Country(
    val isoCode: String = "DE",
    val name: String = "Germany",
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