package net.nymtech.nymvpn.ui.screens.hop

import net.nymtech.nymvpn.model.Countries
import net.nymtech.nymvpn.model.Country

data class HopUiState(
    val loading: Boolean = true,
    val countries: Countries = emptyList(),
    val queriedCountries: Countries = emptyList(),
    val selected: Country? = null,
    val query: String = ""
)