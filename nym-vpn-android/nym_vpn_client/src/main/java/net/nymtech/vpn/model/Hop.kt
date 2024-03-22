package net.nymtech.vpn.model

import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable
import kotlinx.serialization.json.Json
import net.nymtech.vpn.util.Constants
import java.lang.IllegalArgumentException
import java.util.Locale

typealias HopCountries = Set<Hop.Country>
sealed class Hop {
    @Serializable
    data class Country(
        val isoCode: String = Constants.DEFAULT_COUNTRY_ISO,
        val name: String = Locale(isoCode.lowercase(), isoCode).displayCountry,
        val isFastest: Boolean = false,
        val isDefault: Boolean = false
    ) : ExitPoint, EntryPoint, Hop() {

        init {
            if(isoCode.length > 2) {
                throw IllegalArgumentException("isoCode must be two characters")
            }
        }

        override fun toLibString(): String {
            return Location(Location.CountryISO(this.isoCode)).toString()
        }

        override fun toString(): String {
            return Json.encodeToString(serializer(), this)
        }

        companion object {
            //TODO handle errors
            fun from(string: String?): Country {
                return string?.let { Json.decodeFromString<Country>(string)} ?: Country()
            }
            fun fromCollectionString(string: String?) : HopCountries {
                return string?.let {
                    Json.decodeFromString<HopCountries>(it)
                } ?: emptySet()
            }
        }
    }

    @Serializable
    @SerialName("Gateway")
    data class Gateway(@SerialName("identity") val identity : String) : ExitPoint, EntryPoint, Hop() {
        override fun toLibString(): String {
            return this.toString()
        }
    }
}