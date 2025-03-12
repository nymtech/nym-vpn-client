package net.nymtech.nymvpn.util.extensions

import android.annotation.SuppressLint
import android.content.Context
import androidx.compose.runtime.Composable
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.res.vectorResource
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.TextUnit
import androidx.navigation.NavBackStackEntry
import androidx.navigation.NavController
import androidx.navigation.NavDestination.Companion.hasRoute
import androidx.navigation.NavDestination.Companion.hierarchy
import androidx.navigation.NavGraph.Companion.findStartDestination
import net.nymtech.nymvpn.NymVpn
import net.nymtech.nymvpn.ui.Route
import nym_vpn_lib.ErrorStateReason
import nym_vpn_lib.VpnException
import kotlin.reflect.KClass
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.util.Constants
import net.nymtech.vpn.backend.Tunnel
import net.nymtech.vpn.model.NymGateway
import nym_vpn_lib.EntryPoint
import nym_vpn_lib.ExitPoint
import nym_vpn_lib.GatewayType
import nym_vpn_lib.RegisterDeviceError
import nym_vpn_lib.RequestZkNymError
import nym_vpn_lib.Score
import java.util.*

fun Dp.scaledHeight(): Dp {
	return NymVpn.instance.resizeHeight(this)
}

fun Dp.scaledWidth(): Dp {
	return NymVpn.instance.resizeWidth(this)
}

fun TextUnit.scaled(): TextUnit {
	return NymVpn.instance.resizeHeight(this)
}

fun NavController.navigateAndForget(route: Route) {
	navigate(route) {
		popUpTo(graph.startDestinationId) { inclusive = true }
		launchSingleTop = true
	}
}

@SuppressLint("RestrictedApi")
fun <T : Route> NavBackStackEntry?.isCurrentRoute(cls: KClass<T>): Boolean {
	return this?.destination?.hierarchy?.any {
		it.hasRoute(route = cls)
	} == true
}

fun NavController.goFromRoot(route: Route) {
	if (currentBackStackEntry?.isCurrentRoute(route::class) == true) return
	this.navigate(route) {
		// Pop up to the start destination of the graph to
		// avoid building up a large stack of destinations
		// on the back stack as users select items
		popUpTo(graph.findStartDestination().id) {
			saveState = true
		}
		// Avoid multiple copies of the same destination when
		// reselecting the same item
		launchSingleTop = true
		restoreState = true
	}
}

fun ErrorStateReason.toUserMessage(context: Context): String {
	return when (val error = this) {
		ErrorStateReason.SameEntryAndExitGateway -> context.getString(R.string.same_entry_exit_message)
		ErrorStateReason.InvalidEntryGatewayCountry -> context.getString(R.string.selected_entry_unavailable)
		ErrorStateReason.InvalidExitGatewayCountry -> context.getString(R.string.selected_exit_unavailable)
		is ErrorStateReason.RegisterDevice -> {
			when (val registerDeviceError = error.v1) {
				is RegisterDeviceError.ErrorResponse -> {
					if (registerDeviceError.v1.messageId == Constants.MAX_DEVICES_REACHED_ID) {
						context.getString(R.string.max_devices_reached) + " ${context.getString(R.string.remove_device)}"
					} else {
						context.getString(R.string.unexpected_error) + " ${this.javaClass.simpleName}:${registerDeviceError.javaClass.simpleName}"
					}
				}
				else -> context.getString(R.string.unexpected_error) + " ${this.javaClass.simpleName}:${registerDeviceError.javaClass.simpleName}"
			}
		}
		is ErrorStateReason.RequestZkNym -> {
			when (val zknymError = error.v1) {
				is RequestZkNymError.VpnApi -> {
					zknymError.toUserMessage(context, error)
				}
				else -> context.getString(R.string.unexpected_error) + " ${this.javaClass.simpleName}:${zknymError.javaClass.simpleName}"
			}
		}
		is ErrorStateReason.RequestZkNymBundle -> {
			run {
				error.failed.forEach { zkNymError ->
					return@run when (zkNymError) {
						is RequestZkNymError.VpnApi -> {
							zkNymError.toUserMessage(context, error)
						}
						else -> context.getString(R.string.unexpected_error) + " ${error.javaClass.simpleName}:${zkNymError.javaClass.simpleName}"
					}
				}
				context.getString(R.string.unexpected_error) + " ${this.javaClass.simpleName}"
			}
		}
		else -> context.getString(R.string.unexpected_error) + " ${error.javaClass.simpleName}"
	}
}

fun RequestZkNymError.VpnApi.toUserMessage(context: Context, parent: ErrorStateReason): String {
	return when (this.v1.messageId) {
		Constants.MAX_BANDWIDTH_REACHED_ID -> {
			context.getString(R.string.out_of_bandwidth_error)
		}
		Constants.SUBSCRIPTION_EXPIRED_ID -> {
			context.getString(R.string.subscription_expired)
		}
		else -> context.getString(R.string.unexpected_error) + " ${parent.javaClass.simpleName}:${this.javaClass.simpleName}"
	}
}

fun VpnException.toUserMessage(context: Context): String {
	return when (this) {
		is VpnException.NetworkConnectionException -> context.getString(R.string.network_error)
		is VpnException.VpnApiTimeout -> context.getString(R.string.network_error)
		else -> context.getString(R.string.unexpected_error) + " ${this.javaClass.simpleName}"
	}
}

fun List<NymGateway>.scoreSorted(mode: Tunnel.Mode): List<NymGateway> {
	return this.sortedBy {
		when (mode) {
			Tunnel.Mode.FIVE_HOP_MIXNET -> it.mixnetScore
			Tunnel.Mode.TWO_HOP_MIXNET -> it.wgScore
		}
	}
}

fun EntryPoint.Location.toDisplayCountry(): String {
	return Locale(this.location, this.location).displayCountry
}

fun ExitPoint.Location.toDisplayCountry(): String {
	return Locale(this.location, this.location).displayCountry
}

@Composable
fun NymGateway.getScoreIcon(gatewayType: GatewayType): ImageVector {
	val score = when (gatewayType) {
		GatewayType.MIXNET_ENTRY, GatewayType.MIXNET_EXIT -> mixnetScore
		GatewayType.WG -> wgScore
	}
	return when (score) {
		Score.HIGH -> ImageVector.vectorResource(R.drawable.bars_3)
		Score.MEDIUM -> ImageVector.vectorResource(R.drawable.bars_2)
		Score.LOW -> ImageVector.vectorResource(R.drawable.bar_1)
		Score.NONE -> ImageVector.vectorResource(R.drawable.faq)
		null -> ImageVector.vectorResource(R.drawable.faq)
	}
}
