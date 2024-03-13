package net.nymtech.nymvpn.service.gateway.model

import androidx.annotation.Keep
import com.squareup.moshi.Json
import com.squareup.moshi.JsonClass
@Keep
@JsonClass(generateAdapter = true)
data class SelfDescribed(
    @Json(name = "host_information") val hostInformation: HostInformation,
    @Json(name = "build_information") val buildInformation : BuildInformation,
    @Json(name = "ip_packet_router") val ipPacketRouter : IpPacketRouter?,
    @Json(name = "mixnet_websockets") val mixnetWebsockets : MixnetWebsockets
)
