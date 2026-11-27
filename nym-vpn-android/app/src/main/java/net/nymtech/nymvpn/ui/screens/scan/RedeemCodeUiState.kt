package net.nymtech.nymvpn.ui.screens.scan

import androidx.annotation.StringRes

data class RedeemCodeUiState(val code: String = "", val loading: Boolean = false, @StringRes val errorRes: Int? = null, val connectAfter: Boolean = true) {
	override fun toString(): String = "RedeemCodeUiState(code=<redacted>, loading=$loading, errorRes=$errorRes, connectAfter=$connectAfter)"
}
