package net.nymtech.nymvpn.util.extensions

import android.annotation.SuppressLint
import android.content.Context
import android.text.format.DateUtils
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.drawWithContent
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.res.vectorResource
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.TextUnit
import androidx.navigation.NavBackStackEntry
import androidx.navigation.NavController
import androidx.navigation.NavDestination.Companion.hasRoute
import androidx.navigation.NavDestination.Companion.hierarchy
import androidx.navigation.NavGraph.Companion.findStartDestination
import net.nymtech.nymvpn.NymVpn
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.Route
import net.nymtech.nymvpn.ui.screens.main.panel.ConnectMode
import net.nymtech.nymvpn.ui.theme.LocalNymColors
import net.nymtech.vpn.backend.Tunnel
import net.nymtech.vpn.model.BridgeParameter
import net.nymtech.vpn.model.NymGateway
import nym_vpn_lib.VpnException
import nym_vpn_lib_types.EntryPoint
import nym_vpn_lib_types.ErrorStateReason
import nym_vpn_lib_types.ExitPoint
import nym_vpn_lib_types.GatewayType
import nym_vpn_lib_types.Score
import timber.log.Timber
import java.time.Instant
import java.util.Locale
import kotlin.reflect.KClass

fun Dp.scaledHeight(previewScale: Float = 1f): Dp = if (NymVpn.isInitialized) {
	NymVpn.instance.resizeHeight(this)
} else {
	this * previewScale
}

fun Dp.scaledWidth(previewScale: Float = 1f): Dp = if (NymVpn.isInitialized) {
	NymVpn.instance.resizeWidth(this)
} else {
	this * previewScale
}

fun TextUnit.scaled(previewScale: Float = 1f): TextUnit = if (NymVpn.isInitialized) {
	NymVpn.instance.resizeHeight(this)
} else {
	this * previewScale
}

fun Modifier.topBorder(color: Color, height: Dp) = this.drawWithContent {
	drawContent()
	drawLine(
		color = color,
		start = Offset(0f, 0f),
		end = Offset(size.width, 0f),
		strokeWidth = height.toPx(),
	)
}

fun NavController.navigateAndForget(route: Route) {
	if (currentBackStackEntry?.isCurrentRoute(route::class) == true) return
	try {
		navigate(route) {
			popUpTo(graph.findStartDestination().id) { inclusive = true }
			launchSingleTop = true
		}
	} catch (e: Exception) {
		Timber.e("Navigation failed: ${e.message}")
		goFromRoot(Route.Main())
	}
}

fun NavController.replaceCurrentWith(route: Route) {
	val currentRoute = currentBackStackEntry?.destination?.route ?: return
	try {
		navigate(route) {
			popUpTo(currentRoute) { inclusive = true }
			launchSingleTop = true
		}
	} catch (e: Exception) {
		Timber.e("Navigation failed: ${e.message}")
		goFromRoot(Route.Main())
	}
}

@SuppressLint("RestrictedApi")
fun <T : Route> NavBackStackEntry?.isCurrentRoute(cls: KClass<T>): Boolean = this?.destination?.hierarchy?.any {
	it.hasRoute(route = cls)
} == true

fun NavController.goFromRoot(route: Route) {
	if (currentBackStackEntry?.isCurrentRoute(route::class) == true) return
	try {
		this.navigate(route) {
			// Pop up to the start destination of the graph to
			// avoid building up a large stack of destinations
			// on the back stack as users select items
			popUpTo(graph.findStartDestination().id) { saveState = true }
			// Avoid multiple copies of the same destination when
			// reSelecting the same item
			launchSingleTop = true
			restoreState = true
		}
	} catch (e: Exception) {
		Timber.e("Failed to goFromRoot: ${e.message}")
		navigate(Route.Splash)
	}
}

fun NavController.safePopBackStack() {
	if (this.previousBackStackEntry != null) {
		this.popBackStack()
	} else {
		this.goFromRoot(Route.Main())
	}
}

@Suppress("CyclomaticComplexMethod")
fun ErrorStateReason.toUserMessage(context: Context): String = when (this) {
	ErrorStateReason.SameEntryAndExitGateway -> context.getString(R.string.error_same_entry_exit)

	ErrorStateReason.InvalidEntryGatewayCountry -> context.getString(R.string.error_invalid_entry_gateway_country)
	ErrorStateReason.InvalidExitGatewayCountry -> context.getString(R.string.error_invalid_exit_gateway_country)

	ErrorStateReason.BandwidthExceeded -> context.getString(R.string.bandwidth_error)
	ErrorStateReason.MaxDevicesReached -> context.getString(R.string.max_devices_error)
	ErrorStateReason.DeviceTimeOutOfSync -> context.getString(R.string.device_time_out_of_sync)
	ErrorStateReason.Ipv6Unavailable -> context.getString(R.string.error_ipv6_unavailable)
	ErrorStateReason.DeviceLoggedOut -> context.getString(R.string.error_device_logged_out)
	ErrorStateReason.InactiveAccount -> context.getString(R.string.error_inactive_account)

	ErrorStateReason.CredentialFetchingFailed -> context.getString(R.string.error_credential_fetching_failed)
	ErrorStateReason.NoCredentialAvailable -> context.getString(R.string.error_no_credential_available)

	ErrorStateReason.SetDns -> context.getString(R.string.error_set_dns)
	ErrorStateReason.SetFirewallPolicy -> context.getString(R.string.error_set_firewall_policy)
	ErrorStateReason.SetRouting -> context.getString(R.string.error_set_routing)
	ErrorStateReason.TunDevice -> context.getString(R.string.error_tun_device)
	ErrorStateReason.TunnelProvider -> context.getString(R.string.error_tunnel_provider)

	ErrorStateReason.InactiveSubscription -> context.getString(R.string.error_active_plan)

	ErrorStateReason.CredentialWastedOnEntryGateway -> context.getString(R.string.error_bandwidth_entry)
	ErrorStateReason.CredentialWastedOnExitGateway -> context.getString(R.string.error_bandwidth_exit)

	ErrorStateReason.PerformantEntryGatewayUnavailable -> context.getString(R.string.error_gateway_unavailable_entry)
	ErrorStateReason.PerformantExitGatewayUnavailable -> context.getString(R.string.error_gateway_unavailable_exit)

	ErrorStateReason.InvalidEntryGatewayIdentity -> context.getString(R.string.error_invalid_entry_gateway_identity)
	ErrorStateReason.InvalidExitGatewayIdentity -> context.getString(R.string.error_invalid_exit_gateway_identity)

	// unused on Android
	ErrorStateReason.NeedFullDiskPermissions -> ""
	ErrorStateReason.SplitTunnel -> ""

	is ErrorStateReason.Internal -> context.getString(R.string.unexpected_error, this.v1)
	ErrorStateReason.NeedsRelaxedIndependenceCriteria -> context.getString(R.string.node_families_error_message)
}

fun VpnException.toUserMessage(context: Context): String = when (this) {
	is VpnException.NetworkConnectionException -> context.getString(R.string.network_error)
	is VpnException.VpnApiTimeout -> context.getString(R.string.network_error)
	else -> context.getString(R.string.unexpected_error) + " ${this.javaClass.simpleName}"
}

fun List<NymGateway>.scoreSorted(mode: Tunnel.Mode): List<NymGateway> = this.sortedBy {
	when (mode) {
		Tunnel.Mode.FIVE_HOP_MIXNET -> it.mixnetScore ?: Score.OFFLINE
		Tunnel.Mode.TWO_HOP_MIXNET -> it.wgScore ?: Score.OFFLINE
	}
}

fun toDisplayCountry(twoLetterIsoCountryCode: String): String = Locale(twoLetterIsoCountryCode, twoLetterIsoCountryCode).displayCountry

fun EntryPoint.Country.toDisplayCountry(): String = toDisplayCountry(twoLetterIsoCountryCode)

fun ExitPoint.Country.toDisplayCountry(): String = toDisplayCountry(twoLetterIsoCountryCode)

internal fun NymGateway.isQuicSupported(): Boolean = run {
	return bridgeInformation?.transports?.find {
		it is BridgeParameter.QuicPlain
	} != null
}

@Composable
fun NymGateway.getScoreIcon(gatewayType: GatewayType): Pair<ImageVector, String> {
	val score = when (gatewayType) {
		GatewayType.MIXNET_ENTRY, GatewayType.MIXNET_EXIT -> mixnetScore
		GatewayType.WG -> wgScore
	}
	return score?.let {
		getScoreIcon(score)
	} ?: Pair(ImageVector.vectorResource(R.drawable.faq), stringResource(R.string.unknown))
}

@Composable
fun getScoreIcon(score: Score): Pair<ImageVector, String> = when (score) {
	Score.HIGH -> Pair(ImageVector.vectorResource(R.drawable.bars_3), stringResource(R.string.bars_3))
	Score.MEDIUM -> Pair(ImageVector.vectorResource(R.drawable.bars_2), stringResource(R.string.bars_2))
	Score.LOW -> Pair(ImageVector.vectorResource(R.drawable.bar_1), stringResource(R.string.bars_1))
	Score.OFFLINE -> Pair(ImageVector.vectorResource(R.drawable.bar_0), stringResource(R.string.unknown))
}

@Composable
fun Score.colorLoad(): Color = when (this) {
	Score.HIGH -> Color.Red
	Score.MEDIUM -> LocalNymColors.current.warning
	Score.LOW -> Color.Green
	Score.OFFLINE -> Color.Gray
}

@Composable
fun Score.colorPerformance(): Color = when (this) {
	Score.HIGH -> Color.Green
	Score.MEDIUM -> LocalNymColors.current.warning
	Score.LOW -> Color.Red
	Score.OFFLINE -> Color.Gray
}

@Composable
fun Score.displayText(): String = when (this) {
	Score.HIGH -> stringResource(R.string.score_text_high)
	Score.MEDIUM -> stringResource(R.string.score_text_medium)
	Score.LOW -> stringResource(R.string.score_text_low)
	Score.OFFLINE -> stringResource(R.string.score_text_offline)
}

@Composable
fun getModeIcon(mode: ConnectMode): ImageVector = when (mode) {
	ConnectMode.AUTO -> ImageVector.vectorResource(R.drawable.ic_mode_auto)
	ConnectMode.FAST -> ImageVector.vectorResource(R.drawable.ic_mode_fast)
	ConnectMode.MIXNET -> ImageVector.vectorResource(R.drawable.ic_mode_mixnet)
}

fun relativeTimeSpan(utcString: String?): String = try {
	utcString?.let {
		val instant = Instant.parse(it)
		DateUtils.getRelativeTimeSpanString(instant.toEpochMilli()).toString()
	} ?: "--"
} catch (_: Exception) {
	"--"
}

fun isValidIPv4(s: String): Boolean {
	val parts = s.split(".")
	if (parts.size != 4) return false
	return parts.all { p ->
		if (p.isEmpty()) return@all false
		if (p.length > 1 && p.startsWith("0")) return@all false
		val n = p.toIntOrNull() ?: return@all false
		n in 0..255
	}
}

fun isValidIPv6(s: String): Boolean {
	if (s.count { it == ':' } < 2) return false
	if (!s.all { it.isDigit() || it.lowercaseChar() in 'a'..'f' || it == ':' || it == '.' }) return false

	val hasDouble = s.contains("::")
	if (hasDouble && s.indexOf("::") != s.lastIndexOf("::")) return false

	val tail = s.substringAfterLast(':', missingDelimiterValue = "")
	val hasV4Tail = tail.contains('.') && isValidIPv4(tail)

	val groups = s.split(":")
	val nonEmpty = groups.filter { it.isNotEmpty() }

	val maxGroups = if (hasV4Tail) 6 else 8
	if (!hasDouble) {
		if (nonEmpty.size != (if (hasV4Tail) 7 else 8)) return false
	} else {
		if (nonEmpty.size > maxGroups) return false
	}

	val toCheck = if (hasV4Tail) nonEmpty.dropLast(1) else nonEmpty
	return toCheck.all { g ->
		g.length in 1..4 && g.all { it.isDigit() || it.lowercaseChar() in 'a'..'f' }
	}
}
