package net.nymtech.nymvpn.data.model

import net.nymtech.vpn.model.Country
data class Gateways(
    val firstHopCountry: Country = Country(),
    val lastHopCountry: Country = Country(),
    val lowLatencyCountry: Country = Country(),
    val entryCountries: Set<Country> = emptySet(),
    val exitCountries: Set<Country> = emptySet()
)