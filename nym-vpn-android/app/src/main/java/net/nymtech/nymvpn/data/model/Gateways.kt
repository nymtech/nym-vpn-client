package net.nymtech.nymvpn.data.model

import androidx.datastore.preferences.core.stringPreferencesKey
import net.nymtech.vpn.model.Hop
import net.nymtech.vpn.model.HopCountries

data class Gateways(
    val firstHopCountry: Hop.Country = Hop.Country(),
    val lastHopCountry: Hop.Country = Hop.Country(),
    val lowLatencyCountry: Hop.Country = Hop.Country(),
    val entryCountries: HopCountries = emptySet(),
    val exitCountries: HopCountries = emptySet()
)