package net.nymtech.nymvpn.service.gateway.model

import androidx.annotation.Keep
import com.squareup.moshi.Json
import com.squareup.moshi.JsonClass

@Keep
@JsonClass(generateAdapter = true)
data class PledgeAmount(
    @Json(name = "denom") val denom: String,
    @Json(name = "amount") val amount: String,
)
