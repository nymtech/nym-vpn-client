package net.nymtech.nymvpn.service.gateway.model

import com.squareup.moshi.Json
import com.squareup.moshi.JsonClass

@JsonClass(generateAdapter = true)
data class SelfDescribed(
    @Json(name = "host_information") val hostInformation: HostInformation
)
