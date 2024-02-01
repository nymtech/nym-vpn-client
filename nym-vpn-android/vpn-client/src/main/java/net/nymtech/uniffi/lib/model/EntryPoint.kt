package net.nymtech.uniffi.lib.model

import kotlinx.serialization.Serializable
import kotlinx.serialization.json.Json

@Serializable
sealed class EntryPoint {

    @Serializable
    data class Gateway(
        val identity: String
    ) : EntryPoint() {
        companion object
    }

    @Serializable
    data class Location(
        val location: String
    ) : EntryPoint() {
        companion object
    }

    override fun toString(): String {
        return Json.encodeToString(serializer(), this)
    }
}