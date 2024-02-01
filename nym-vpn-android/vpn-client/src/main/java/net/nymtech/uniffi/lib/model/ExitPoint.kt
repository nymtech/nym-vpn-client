package net.nymtech.uniffi.lib.model

import kotlinx.serialization.Serializable
import kotlinx.serialization.json.Json

@Serializable
sealed class ExitPoint {
    @Serializable
    data class Address(

        val address: String
    ) : ExitPoint()

    @Serializable
    data class Gateway(

        val identity: String
    ) : ExitPoint()

    @Serializable
    data class Location(

        val location: String
    ) : ExitPoint()

    override fun toString(): String {
        return Json.encodeToString(serializer(), this)
    }
}