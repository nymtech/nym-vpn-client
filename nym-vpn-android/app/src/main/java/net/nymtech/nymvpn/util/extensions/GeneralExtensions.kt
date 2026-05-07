package net.nymtech.nymvpn.util.extensions

import android.content.Context
import android.provider.Settings
import net.nymtech.nymvpn.ui.screens.main.components.PanelState
import net.nymtech.vpn.model.NymGateway
import nym_vpn_lib_types.GatewaySelectionAlgorithm
import java.util.Locale
import kotlin.math.round
import kotlin.time.Duration
import kotlin.time.Duration.Companion.seconds

fun Long.convertSecondsToTimeString(): String {
	val duration: Duration = seconds
	return duration.toComponents { hour, minute, second, _ ->
		"%02d:%02d:%02d".format(hour, minute, second)
	}
}

fun String.capitalize(locale: Locale): String = this.replaceFirstChar { if (it.isLowerCase()) it.titlecase(locale) else it.toString() }

fun Long.toMB(): String {
	val mb = this / 1024.0 * 1024.0
	return "%.2f".format(round(mb * 100) / 100)
}

fun String.truncateWithEllipsis(length: Int): String = if (this.length <= length) this else "${take(length)}..."

fun NymGateway.toLocale(): Locale? = twoLetterCountryISO?.let { Locale(it, it) }

private const val ALWAYS_ON_VPN_APP = "always_on_vpn_app"

fun isVpnAlwaysOn(context: Context): Boolean = try {
	val alwaysOn = Settings.Secure.getString(context.contentResolver, ALWAYS_ON_VPN_APP)
	alwaysOn == context.packageName
} catch (ex: SecurityException) {
	false
}

fun PanelState.toAlgorithm(): GatewaySelectionAlgorithm = when (this) {
	PanelState.COLLAPSED -> GatewaySelectionAlgorithm.AUTO
	PanelState.MODE -> GatewaySelectionAlgorithm.AUTO_ENTRY_EXPLICIT_EXIT
	PanelState.FULL -> GatewaySelectionAlgorithm.EXPLICIT
}

fun GatewaySelectionAlgorithm.toPanelState(): PanelState = when (this) {
	GatewaySelectionAlgorithm.AUTO -> PanelState.COLLAPSED
	GatewaySelectionAlgorithm.AUTO_ENTRY_EXPLICIT_EXIT -> PanelState.MODE
	GatewaySelectionAlgorithm.EXPLICIT -> PanelState.FULL
}
