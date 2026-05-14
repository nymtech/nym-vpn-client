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
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.TextUnit
import androidx.core.net.toUri
import androidx.credentials.CreatePasswordRequest
import androidx.credentials.CredentialManager
import net.nymtech.nymvpn.NymVpn.Companion.instance
import net.nymtech.nymvpn.service.android.tile.VpnQuickTile
import net.nymtech.nymvpn.util.Constants
import timber.log.Timber

private const val BASELINE_HEIGHT = 2201
private const val BASELINE_WIDTH = 1080
private const val BASELINE_DENSITY = 2.625

val Context.actionBarSize
	get() = theme.obtainStyledAttributes(intArrayOf(android.R.attr.actionBarSize))
		.let { attrs -> attrs.getDimension(0, 0F).toInt().also { attrs.recycle() } }

fun Context.openWebUrl(url: String): Result<Unit> = kotlin.runCatching {
	val webpage: Uri = Uri.parse(url)
	Intent(Intent.ACTION_VIEW, webpage).apply {
		addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
	}.also {
		startActivity(it)
	}
}

fun Context.showToast(resId: Int) {
	Toast.makeText(
		this,
		this.getString(resId),
		Toast.LENGTH_LONG,
	).show()
}

fun Context.launchVpnSettings(): Result<Unit> = kotlin.runCatching {
	val intent = Intent(Constants.VPN_SETTINGS_PACKAGE).apply {
		setFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
	}
	startActivity(intent)
}

@SuppressLint("DiscouragedApi")
fun Context.getFlagImageVectorByName(name: String): Int {
	val flagAssetName = "flag_%S".format(name).lowercase()
	val resourceId =
		resources.getIdentifier(flagAssetName, "drawable", packageName)
	return if (resourceId == 0) {
		Timber.e("Cannot find flag for countryIso: $name")
		// use our unknown flag drawable
		resources.getIdentifier("flag_unknown", "drawable", packageName)
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

fun Context.launchBatteryOptSettingsScreen() {
	val packageName = "package:${this.packageName}".toUri()
	val intent = Intent(Settings.ACTION_APPLICATION_DETAILS_SETTINGS).apply {
		data = packageName
	}
	this.startActivity(intent)
}

fun Context.launchPrivateDnsSettings() {
	if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q) {
		val intent = Intent("android.settings.PRIVATE_DNS_SETTINGS").apply {
			addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
		}
		try {
			startActivity(intent)
		} catch (e: android.content.ActivityNotFoundException) {
			startActivity(
				Intent(Settings.ACTION_WIRELESS_SETTINGS).apply {
					addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
				},
			)
		}
	}
}

fun Context.isPrivateDnsEnabled(): Boolean {
	if (Build.VERSION.SDK_INT < Build.VERSION_CODES.Q) return false
	val mode = Settings.Global.getString(contentResolver, "private_dns_mode")
	return mode != "off"
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

fun Context.isAndroidTV(): Boolean = packageManager.hasSystemFeature(PackageManager.FEATURE_LEANBACK)

suspend fun savePasswordToManager(context: Context, password: String) {
	val credentialManager = CredentialManager.create(context)
	val passwordCredential = CreatePasswordRequest(id = "nym-passphrase", password = password)
	try {
		credentialManager.createCredential(request = passwordCredential, context = context)
	} catch (e: Exception) {
		Timber.d(e)
	}
}
