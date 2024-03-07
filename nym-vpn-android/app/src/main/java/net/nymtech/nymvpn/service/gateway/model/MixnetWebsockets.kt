package net.nymtech.nymvpn.service.gateway.model

import com.squareup.moshi.Json

data class MixnetWebsockets(
    @Json(name = "ws_port") val wsPort : Int?,
    @Json(name = "wss_port") val wssPort : Int?
)