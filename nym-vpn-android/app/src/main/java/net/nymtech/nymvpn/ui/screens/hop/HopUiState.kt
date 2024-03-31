package net.nymtech.nymvpn.ui.screens.hop

import net.nymtech.nymvpn.ui.HopType
import net.nymtech.vpn.model.Country

data class HopUiState(
    val countries: Set<Country> = emptySet(),
    val hopType: HopType = HopType.FIRST,
    val queriedCountries: Set<Country> = emptySet(),
    val selected: Country? = null,
    val query: String = ""
)