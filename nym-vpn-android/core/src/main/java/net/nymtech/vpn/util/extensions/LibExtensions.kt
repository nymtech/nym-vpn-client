package net.nymtech.vpn.util.extensions

import android.content.Context
import net.nymtech.vpn.R
import net.nymtech.vpn.backend.Tunnel
import net.nymtech.vpn.util.Base58
import nym_vpn_lib_types.EntryPoint
import nym_vpn_lib_types.ErrorStateReason
import nym_vpn_lib_types.ExitPoint
import nym_vpn_lib_types.FavoriteSelector
import nym_vpn_lib_types.TunnelEvent
import nym_vpn_lib_types.TunnelState
import java.util.Locale

fun TunnelEvent.NewState.asTunnelState(): Tunnel.State = when (this.v1) {
	is TunnelState.Connected -> Tunnel.State.Up
	is TunnelState.Connecting -> Tunnel.State.EstablishingConnection
	is TunnelState.Disconnected -> Tunnel.State.Down
	is TunnelState.Disconnecting -> Tunnel.State.Disconnecting
	is TunnelState.Error -> Tunnel.State.Error(this.v1.v1)
	is TunnelState.Offline -> Tunnel.State.Offline
}

enum class GatewaySelectionMode(val value: String) {
	RANDOM("Random"),
	AUTO("Auto"),
}

fun EntryPoint.asString(): String = when (val entry = this) {
	is EntryPoint.Gateway -> entry.identity
	is EntryPoint.Country -> entry.twoLetterIsoCountryCode.lowercase()
	EntryPoint.Random -> GatewaySelectionMode.RANDOM.value
	is EntryPoint.Region -> entry.region
	is EntryPoint.Auto -> GatewaySelectionMode.AUTO.value
}

fun ExitPoint.asString(): String = when (val exit = this) {
	is ExitPoint.Gateway -> exit.identity
	is ExitPoint.Country -> exit.twoLetterIsoCountryCode.lowercase()
	is ExitPoint.Address -> exit.address
	is ExitPoint.Random -> GatewaySelectionMode.RANDOM.value
	is ExitPoint.Region -> exit.region
	is ExitPoint.Auto -> GatewaySelectionMode.AUTO.value
}

fun String.asEntryPoint(): EntryPoint = when {
	this == GatewaySelectionMode.RANDOM.value -> EntryPoint.Random
	this == GatewaySelectionMode.AUTO.value -> EntryPoint.Auto(excludeUserCountry = true)
	length == 2 -> EntryPoint.Country(this.uppercase())
	Base58.isValidBase58(this, 32) -> EntryPoint.Gateway(this)
	else -> EntryPoint.Region(this)
}

fun String.asExitPoint(): ExitPoint = when {
	length == 2 -> ExitPoint.Country(this.uppercase())
	length == 134 -> ExitPoint.Address(this)
	Base58.isValidBase58(this, 32) -> ExitPoint.Gateway(this)
	this == GatewaySelectionMode.RANDOM.value -> ExitPoint.Random
	this == GatewaySelectionMode.AUTO.value -> ExitPoint.Auto(excludeEntryPointCountry = true, excludeUserCountry = true)
	else -> ExitPoint.Region(this)
}

fun String.asFavoriteSelector(): FavoriteSelector = when {
	length == 2 -> FavoriteSelector.Country(this.uppercase())
	Base58.isValidBase58(this, 32) -> FavoriteSelector.Gateway(this)
	else -> FavoriteSelector.Region(this)
}

fun toDisplayCountry(twoLetterIsoCountryCode: String): String = Locale(twoLetterIsoCountryCode, twoLetterIsoCountryCode).displayCountry

private val ERROR_STATE_REASON_STRING_RES: Map<ErrorStateReason, Int> = mapOf(
	ErrorStateReason.SetFirewallPolicy to R.string.error_reason_set_firewall_policy,
	ErrorStateReason.SetRouting to R.string.error_reason_set_routing,
	ErrorStateReason.SetDns to R.string.error_reason_set_dns,
	ErrorStateReason.TunDevice to R.string.error_reason_tun_device,
	ErrorStateReason.TunnelProvider to R.string.error_reason_tunnel_provider,
	ErrorStateReason.Ipv6Unavailable to R.string.error_reason_ipv6_unavailable,
	ErrorStateReason.SameEntryAndExitGateway to R.string.error_reason_same_entry_and_exit_gateway,
	ErrorStateReason.PerformantEntryGatewayUnavailable to R.string.error_reason_performant_entry_gateway_unavailable,
	ErrorStateReason.PerformantExitGatewayUnavailable to R.string.error_reason_performant_exit_gateway_unavailable,
	ErrorStateReason.InvalidEntryGatewayIdentity to R.string.error_reason_invalid_entry_gateway_identity,
	ErrorStateReason.InvalidExitGatewayIdentity to R.string.error_reason_invalid_exit_gateway_identity,
	ErrorStateReason.InvalidEntryGatewayCountry to R.string.error_reason_invalid_entry_gateway_country,
	ErrorStateReason.InvalidExitGatewayCountry to R.string.error_reason_invalid_exit_gateway_country,
	ErrorStateReason.CredentialWastedOnEntryGateway to R.string.error_reason_credential_wasted_on_entry_gateway,
	ErrorStateReason.CredentialWastedOnExitGateway to R.string.error_reason_credential_wasted_on_exit_gateway,
	ErrorStateReason.BandwidthExceeded to R.string.error_reason_bandwidth_exceeded,
	ErrorStateReason.InactiveAccount to R.string.error_reason_inactive_account,
	ErrorStateReason.InactiveSubscription to R.string.error_reason_inactive_subscription,
	ErrorStateReason.MaxDevicesReached to R.string.error_reason_max_devices_reached,
	ErrorStateReason.DeviceTimeOutOfSync to R.string.error_reason_device_time_out_of_sync,
	ErrorStateReason.DeviceLoggedOut to R.string.error_reason_device_logged_out,
	ErrorStateReason.CredentialFetchingFailed to R.string.error_reason_credential_fetching_failed,
	ErrorStateReason.NoCredentialAvailable to R.string.error_reason_no_credential_available,
	ErrorStateReason.NeedsDeviceLocation to R.string.error_reason_needs_device_location,
)

fun ErrorStateReason.toHumanReadableString(context: Context): String = when (this) {
	is ErrorStateReason.Internal -> context.getString(R.string.error_reason_internal, this.v1)
	// unused on Android
	ErrorStateReason.NeedFullDiskPermissions, ErrorStateReason.SplitTunnel, ErrorStateReason.NeedsRelaxedIndependenceCriteria -> ""
	else -> ERROR_STATE_REASON_STRING_RES[this]?.let { context.getString(it) } ?: ""
}
