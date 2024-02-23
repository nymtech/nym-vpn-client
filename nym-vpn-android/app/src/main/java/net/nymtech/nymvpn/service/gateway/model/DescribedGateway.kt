package net.nymtech.nymvpn.service.gateway.model

import com.squareup.moshi.Json
import com.squareup.moshi.JsonClass


@JsonClass(generateAdapter = true)
data class DescribedGateway(
    @Json(name = "bond") val bond: Bond,
    @Json(name = "self_described") val selfDescribed: SelfDescribed
)
