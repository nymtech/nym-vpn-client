package net.nymtech.nymvpn.ui.screens.main.profiles

import androidx.annotation.DrawableRes
import androidx.annotation.StringRes
import net.nymtech.nymvpn.R
import nym_vpn_lib_types.Profile as CoreProfile

enum class Profile(@DrawableRes val icon: Int, @StringRes val titleRes: Int, @StringRes val descriptionRes: Int) {
	SAFEST(R.drawable.ic_safest, R.string.profiles_safest_title, R.string.profiles_safest_description),
	RANDOM(R.drawable.ic_random, R.string.profiles_random_title, R.string.profiles_random_description),
	MOST_PRIVATE(R.drawable.ic_private, R.string.profiles_most_private_title, R.string.profiles_most_private_description),
	FASTEST(R.drawable.ic_fastest, R.string.profiles_fastest_title, R.string.profiles_fastest_description),
}

fun Profile.toCoreProfile(): CoreProfile = when (this) {
	Profile.SAFEST -> CoreProfile.SAFEST
	Profile.MOST_PRIVATE -> CoreProfile.MOST_PRIVATE
	Profile.FASTEST -> CoreProfile.FASTEST
	Profile.RANDOM -> CoreProfile.RANDOM
}
