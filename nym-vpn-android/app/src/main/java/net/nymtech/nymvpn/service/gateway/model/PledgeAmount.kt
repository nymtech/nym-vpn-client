package net.nymtech.nymvpn.service.gateway.model

import com.squareup.moshi.Json
import com.squareup.moshi.JsonClass

@JsonClass(generateAdapter = true)
data class PledgeAmount(
    @Json(name = "denom") val denom: String,
    @Json(name = "amount") val amount: String,
)
