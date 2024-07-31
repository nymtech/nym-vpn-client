package net.nymtech.nymvpn.util.extensions

import android.annotation.SuppressLint
import android.content.Context
import android.content.Intent
import android.net.Uri
import android.widget.Toast
import androidx.compose.ui.text.buildAnnotatedString
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.TextUnit
import androidx.core.os.LocaleListCompat
import androidx.navigation.NavController
import net.nymtech.nymvpn.NymVpn
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.util.Constants
import net.nymtech.vpn.model.Country
import timber.log.Timber
import java.util.Locale

fun Dp.scaledHeight(): Dp {
	return NymVpn.resizeHeight(this)
}

fun Dp.scaledWidth(): Dp {
	return NymVpn.resizeWidth(this)
}

fun TextUnit.scaled(): TextUnit {
	return NymVpn.resizeHeight(this)
}

val Context.actionBarSize
	get() = theme.obtainStyledAttributes(intArrayOf(android.R.attr.actionBarSize))
		.let { attrs -> attrs.getDimension(0, 0F).toInt().also { attrs.recycle() } }

fun Context.buildCountryNameString(country: Country): String {
	return buildAnnotatedString {
		if (country.isLowLatency) {
			append(getString(R.string.fastest))
			append(" (")
			append(country.name)
			append(")")
		} else {
			append(country.name)
		}
	}.text
}

fun Context.openWebUrl(url: String): Result<Unit> {
	return kotlin.runCatching {
		val webpage: Uri = Uri.parse(url)
		Intent(Intent.ACTION_VIEW, webpage).apply {
			addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
		}.also {
			startActivity(it)
		}
	}
}

fun Context.showToast(resId: Int) {
	Toast.makeText(
		this,
		this.getString(resId),
		Toast.LENGTH_LONG,
	).show()
}

fun Context.launchVpnSettings(): Result<Unit> {
	return kotlin.runCatching {
		val intent = Intent(Constants.VPN_SETTINGS_PACKAGE).apply {
			setFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
		}
		startActivity(intent)
	}
}

@SuppressLint("DiscouragedApi")
fun Context.getFlagImageVectorByName(name: String): Int {
	val flagAssetName = "flag_%S".format(name).lowercase()
	val resourceId =
		resources.getIdentifier(flagAssetName, "drawable", packageName)
	return if (resourceId == 0) {
		Timber.e("Cannot find flag for countryIso: $name")
		// use our unknown flag drawable
		return resources.getIdentifier("flag_unknown", "drawable", packageName)
	} else {
		resourceId
	}
}

fun NavController.navigateAndForget(route: String) {
	navigate(route) {
		popUpTo(0)
	}
}

fun LocaleListCompat.toSet(): Set<Locale> {
	val set = HashSet<Locale>()
	var counter = 0
	while (this[counter] != null) {
		this[counter]?.let { set.add(it) }
		counter++
	}
	return set
}
