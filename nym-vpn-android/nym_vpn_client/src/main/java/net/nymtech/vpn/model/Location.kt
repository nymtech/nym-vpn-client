package net.nymtech.vpn.model

import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable
import kotlinx.serialization.json.Json

@Serializable
internal data class Location(@SerialName("Location") val countryISO: CountryISO) {
    @Serializable
    internal class CountryISO(@SerialName("location") val iso : String)
    override fun toString(): String {
        return Json.encodeToString(serializer(), this)
    }
}
