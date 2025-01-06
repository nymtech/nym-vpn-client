package net.nymtech.nymvpn.util.extensions

import android.annotation.SuppressLint
import android.app.Activity
import android.content.ComponentName
import android.content.Context
import android.content.Intent
import android.content.pm.PackageManager
import android.net.Uri
import android.os.Build
import android.provider.Settings
import android.service.quicksettings.TileService
import android.widget.Toast
import androidx.compose.ui.text.buildAnnotatedString
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.TextUnit
import net.nymtech.nymvpn.BuildConfig
import net.nymtech.nymvpn.NymVpn
import net.nymtech.nymvpn.NymVpn.Companion.instance
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.service.android.tile.VpnQuickTile
import net.nymtech.nymvpn.util.Constants
import net.nymtech.vpn.model.Country
import nym_vpn_lib.UserAgent
import timber.log.Timber
import java.util.Locale

private const val BASELINE_HEIGHT = 2201
private const val BASELINE_WIDTH = 1080
private const val BASELINE_DENSITY = 2.625

val Context.actionBarSize
	get() = theme.obtainStyledAttributes(intArrayOf(android.R.attr.actionBarSize))
		.let { attrs -> attrs.getDimension(0, 0F).toInt().also { attrs.recycle() } }

fun Context.buildCountryNameString(country: Country): String {
	return buildAnnotatedString {
		if (country.isLowLatency) {
			append(getString(R.string.automatic))
		} else {
			append(country.name)
		}
	}.text
}

fun Context.setAppLocale(locale: Locale): Context {
	Locale.setDefault(locale)
	val config = resources.configuration
	config.setLocale(locale)
	config.setLayoutDirection(locale)
	return createConfigurationContext(config)
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

fun Context.resizeHeight(dp: Dp): Dp {
	val displayMetrics = resources.displayMetrics
	val density = displayMetrics.density
	val height = displayMetrics.heightPixels - instance.actionBarSize
	val resizeHeightPercentage =
		(height.toFloat() / BASELINE_HEIGHT) * (BASELINE_DENSITY.toFloat() / density)
	return dp * resizeHeightPercentage
}

fun Context.resizeHeight(textUnit: TextUnit): TextUnit {
	val displayMetrics = resources.displayMetrics
	val density = displayMetrics.density
	val height = displayMetrics.heightPixels - instance.actionBarSize
	val resizeHeightPercentage =
		(height.toFloat() / BASELINE_HEIGHT) * (BASELINE_DENSITY.toFloat() / density)
	return textUnit * resizeHeightPercentage * 1.1
}

fun Context.resizeWidth(dp: Dp): Dp {
	val displayMetrics = resources.displayMetrics
	val density = displayMetrics.density
	val width = displayMetrics.widthPixels
	val resizeWidthPercentage =
		(width.toFloat() / BASELINE_WIDTH) * (BASELINE_DENSITY.toFloat() / density)
	return dp * resizeWidthPercentage
}

fun Context.requestTileServiceStateUpdate() {
	TileService.requestListeningState(
		this,
		ComponentName(instance, VpnQuickTile::class.java),
	)
}

fun Context.launchShareFile(file: Uri) {
	val shareIntent = Intent().apply {
		setAction(Intent.ACTION_SEND)
		setType("*/*")
		putExtra(Intent.EXTRA_STREAM, file)
		addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
	}
	this.startActivity(Intent.createChooser(shareIntent, ""))
}

fun Context.launchNotificationSettings() {
	if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
		val settingsIntent: Intent = Intent(Settings.ACTION_APP_NOTIFICATION_SETTINGS)
			.addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
			.putExtra(Settings.EXTRA_APP_PACKAGE, packageName)
		this.startActivity(settingsIntent)
	} else {
		this.launchAppSettings()
	}
}

// for localization changes
fun Activity.resetTile() {
	try {
		val label = packageManager.getActivityInfo(componentName, PackageManager.GET_META_DATA).labelRes
		if (label != 0) {
			setTitle(label)
		}
	} catch (e: PackageManager.NameNotFoundException) {
		Timber.e(e)
	}
}

fun Context.launchAppSettings() {
	kotlin.runCatching {
		val intent = Intent(Settings.ACTION_APPLICATION_DETAILS_SETTINGS).apply {
			data = Uri.fromParts("package", packageName, null)
			setFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
		}
		startActivity(intent)
	}
}

fun Context.isAndroidTV(): Boolean {
	return packageManager.hasSystemFeature(PackageManager.FEATURE_LEANBACK)
}

fun Context.toUserAgent(): UserAgent {
	val platform = if (isAndroidTV()) "AndroidTV" else "Android"
	return UserAgent(
		Constants.APP_PROJECT_NAME,
		BuildConfig.VERSION_NAME,
		"$platform; ${Build.VERSION.SDK_INT}; ${NymVpn.getCPUArchitecture()}; ${BuildConfig.FLAVOR}",
		BuildConfig.COMMIT_HASH,
	)
}
