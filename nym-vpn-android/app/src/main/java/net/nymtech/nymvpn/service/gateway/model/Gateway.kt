package net.nymtech.nymvpn.service.gateway.model

import com.squareup.moshi.Json
import com.squareup.moshi.JsonClass

@JsonClass(generateAdapter = true)
data class Gateway(
    @Json(name = "host") val host: String,
    @Json(name = "mix_port") val mixPort: Int,
    @Json(name = "clients_port") val clientsPort: Int,
    @Json(name = "location") val location: String,
    @Json(name = "sphinx_key") val sphinxKey: String,
    @Json(name = "identity_key") val identityKey: String,
    @Json(name = "version") val version: String
)
