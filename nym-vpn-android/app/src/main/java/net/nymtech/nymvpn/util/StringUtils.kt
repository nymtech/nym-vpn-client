package net.nymtech.nymvpn.util

import android.annotation.SuppressLint
import android.content.Context
import androidx.compose.ui.text.buildAnnotatedString
import net.nymtech.nymvpn.R
import net.nymtech.vpn.model.Country
import timber.log.Timber

object StringUtils {
	fun buildCountryNameString(country: Country, context: Context): String {
		return buildAnnotatedString {
			if (country.isLowLatency) {
				append(context.getString(R.string.fastest))
				append(" (")
				append(country.name)
				append(")")
			} else {
				append(country.name)
			}
		}.text
	}

	@SuppressLint("DiscouragedApi")
	fun getFlagImageVectorByName(context: Context, name: String): Int {
		val flagAssetName = "flag_%S".format(name).lowercase()
		val resourceId =
			context.resources.getIdentifier(flagAssetName, "drawable", context.packageName)
		return if (resourceId == 0) {
			// TODO add a unknown icon flag
			Timber.e("Cannot find flag for countryIso: $name")
			0
		} else {
			resourceId
		}
	}
}
