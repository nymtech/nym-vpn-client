package net.nymtech.nymvpn.service.gateway.model

import com.squareup.moshi.Json

data class IpPacketRouter(
    @Json(name = "address") val address : String
)