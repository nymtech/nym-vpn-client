package net.nymtech.nymvpn.service.gateway.domain

import androidx.annotation.Keep
import com.squareup.moshi.Json
import com.squareup.moshi.JsonClass

@Keep
@JsonClass(generateAdapter = true)
data class Location(
	@Json(name = "two_letter_ios_country_code") val twoLetterIsoCountryCode: String,
	@Json(name = "latitude") val latitude: Float,
	@Json(name = "longitude") val longitude: Float,
)
