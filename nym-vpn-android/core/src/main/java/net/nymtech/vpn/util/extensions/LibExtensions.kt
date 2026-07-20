package net.nymtech.vpn.util.extensions

import android.content.Context
import net.nymtech.vpn.R
import net.nymtech.vpn.backend.Tunnel
import net.nymtech.vpn.util.Base58
import nym_vpn_lib_types.EntryPoint
import nym_vpn_lib_types.ErrorStateReason
import nym_vpn_lib_types.ExitPoint
import nym_vpn_lib_types.GatewaySelectionAlgorithm
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

fun EntryPoint.asString(): String = when (val entry = this) {
	is EntryPoint.Gateway -> entry.identity
	is EntryPoint.Country -> entry.twoLetterIsoCountryCode.lowercase()
	EntryPoint.Random -> "Random"
	is EntryPoint.Region -> entry.region.lowercase()
}

fun ExitPoint.asString(): String = when (val exit = this) {
	is ExitPoint.Gateway -> exit.identity
	is ExitPoint.Country -> exit.twoLetterIsoCountryCode.lowercase()
	is ExitPoint.Address -> exit.address
	is ExitPoint.Random -> "Random"
	is ExitPoint.Region -> exit.region
}

fun String.asEntryPoint(): EntryPoint = when {
	this == "Random" -> EntryPoint.Random
	length == 2 -> EntryPoint.Country(this.uppercase())
	Base58.isValidBase58(this, 32) -> EntryPoint.Gateway(this)
	else -> EntryPoint.Region(this)
}

fun String.asExitPoint(): ExitPoint = when {
	length == 2 -> ExitPoint.Country(this.uppercase())
	length == 134 -> ExitPoint.Address(this)
	Base58.isValidBase58(this, 32) -> ExitPoint.Gateway(this)
	this == "Random" -> ExitPoint.Random
	else -> throw IllegalArgumentException("Invalid exit id $this")
}

fun String.asAlgorithm(): GatewaySelectionAlgorithm = when {
	this == "AUTO" -> GatewaySelectionAlgorithm.AUTO
	this == "EXPLICIT" -> GatewaySelectionAlgorithm.EXPLICIT
	this == "AUTO_ENTRY_EXPLICIT_EXIT" -> GatewaySelectionAlgorithm.AUTO_ENTRY_EXPLICIT_EXIT
	else -> throw IllegalArgumentException("Invalid GatewaySelectionAlgorithm $this")
}

fun toDisplayCountry(twoLetterIsoCountryCode: String): String = Locale(twoLetterIsoCountryCode, twoLetterIsoCountryCode).displayCountry

fun ErrorStateReason.toHumanReadableString(context: Context): String = when (this) {
	ErrorStateReason.SetFirewallPolicy -> context.getString(R.string.error_reason_set_firewall_policy)
	ErrorStateReason.SetRouting -> context.getString(R.string.error_reason_set_routing)
	ErrorStateReason.SetDns -> context.getString(R.string.error_reason_set_dns)
	ErrorStateReason.TunDevice -> context.getString(R.string.error_reason_tun_device)
	ErrorStateReason.TunnelProvider -> context.getString(R.string.error_reason_tunnel_provider)
	ErrorStateReason.Ipv6Unavailable -> context.getString(R.string.error_reason_ipv6_unavailable)
	ErrorStateReason.SameEntryAndExitGateway -> context.getString(R.string.error_reason_same_entry_and_exit_gateway)
	ErrorStateReason.PerformantEntryGatewayUnavailable -> context.getString(R.string.error_reason_performant_entry_gateway_unavailable)
	ErrorStateReason.PerformantExitGatewayUnavailable -> context.getString(R.string.error_reason_performant_exit_gateway_unavailable)
	ErrorStateReason.InvalidEntryGatewayIdentity -> context.getString(R.string.error_reason_invalid_entry_gateway_identity)
	ErrorStateReason.InvalidExitGatewayIdentity -> context.getString(R.string.error_reason_invalid_exit_gateway_identity)
	ErrorStateReason.InvalidEntryGatewayCountry -> context.getString(R.string.error_reason_invalid_entry_gateway_country)
	ErrorStateReason.InvalidExitGatewayCountry -> context.getString(R.string.error_reason_invalid_exit_gateway_country)
	ErrorStateReason.CredentialWastedOnEntryGateway -> context.getString(R.string.error_reason_credential_wasted_on_entry_gateway)
	ErrorStateReason.CredentialWastedOnExitGateway -> context.getString(R.string.error_reason_credential_wasted_on_exit_gateway)
	ErrorStateReason.BandwidthExceeded -> context.getString(R.string.error_reason_bandwidth_exceeded)
	ErrorStateReason.InactiveAccount -> context.getString(R.string.error_reason_inactive_account)
	ErrorStateReason.InactiveSubscription -> context.getString(R.string.error_reason_inactive_subscription)
	ErrorStateReason.MaxDevicesReached -> context.getString(R.string.error_reason_max_devices_reached)
	ErrorStateReason.DeviceTimeOutOfSync -> context.getString(R.string.error_reason_device_time_out_of_sync)
	ErrorStateReason.DeviceLoggedOut -> context.getString(R.string.error_reason_device_logged_out)
	ErrorStateReason.CredentialFetchingFailed -> context.getString(R.string.error_reason_credential_fetching_failed)
	ErrorStateReason.NoCredentialAvailable -> context.getString(R.string.error_reason_no_credential_available)

	// unused on Android
	ErrorStateReason.NeedFullDiskPermissions, ErrorStateReason.SplitTunnel -> ""

	is ErrorStateReason.Internal -> context.getString(R.string.error_reason_internal, this.v1)
	ErrorStateReason.NeedsRelaxedIndependenceCriteria -> ""
}
