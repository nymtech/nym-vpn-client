package net.nymtech.nymvpn.service.gateway.model

import androidx.annotation.Keep
import com.squareup.moshi.Json
import com.squareup.moshi.JsonClass

@Keep
@JsonClass(generateAdapter = true)
data class HostInformation(
    @Json(name = "ip_address") val ipAddress: List<String>,
    @Json(name = "hostname") val hostname: String,
    @Json(name = "keys") val keys: Keys
)
