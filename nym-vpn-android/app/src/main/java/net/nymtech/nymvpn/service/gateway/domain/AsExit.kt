package net.nymtech.nymvpn.service.gateway.domain

import androidx.annotation.Keep
import com.squareup.moshi.Json
import com.squareup.moshi.JsonClass

@Keep
@JsonClass(generateAdapter = true)
class AsExit(
	@Json(name = "can_connect") val canConnect: Boolean,
	@Json(name = "can_route_ip_external_v4") val canRouteIpv4External: Boolean,
	@Json(name = "can_route_ip_external_v6") val canRouteIpv6External: Boolean,
	@Json(name = "can_route_ip_v4") val canRouteIpv4: Boolean,
	@Json(name = "can_route_ip_v6") val canRouteIpv6: Boolean,
)
