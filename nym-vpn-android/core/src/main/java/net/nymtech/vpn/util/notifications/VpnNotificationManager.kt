package net.nymtech.vpn.util.notifications

import android.Manifest
import android.annotation.SuppressLint
import android.app.Notification
import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.PendingIntent
import android.content.Context
import android.content.Intent
import android.content.pm.PackageManager
import android.graphics.Color
import android.os.Build
import androidx.core.app.ActivityCompat
import androidx.core.app.NotificationCompat
import androidx.core.app.NotificationManagerCompat
import net.nymtech.vpn.R
import net.nymtech.vpn.backend.Tunnel
import net.nymtech.vpn.model.NymGateway
import net.nymtech.vpn.util.SingletonHolder
import net.nymtech.vpn.util.extensions.GatewaySelectionMode
import net.nymtech.vpn.util.extensions.toDisplayCountry
import net.nymtech.vpn.util.extensions.toHumanReadableString
import nym_vpn_lib_types.EntryPoint
import nym_vpn_lib_types.ExitPoint

/**
 * Builds and updates the VPN foreground notification.
 */
@SuppressLint("MissingPermission")
internal class VpnNotificationManager private constructor(private val context: Context) {

	companion object : SingletonHolder<VpnNotificationManager, Context>(::VpnNotificationManager) {
		const val VPN_CHANNEL_ID = "VpnForegroundChannel"
		const val VPN_FOREGROUND_ID = 223
	}

	/** Runs [action] only if POST_NOTIFICATIONS is granted (Android 13+). */
	inline fun withNotificationPermission(action: () -> Unit) {
		if (
			ActivityCompat.checkSelfPermission(
				context,
				Manifest.permission.POST_NOTIFICATIONS,
			) == PackageManager.PERMISSION_GRANTED
		) {
			action()
		}
	}

	private fun setupChannel() {
		if (Build.VERSION.SDK_INT < Build.VERSION_CODES.O) return

		val channel = NotificationChannel(
			VPN_CHANNEL_ID,
			context.getString(R.string.channel_name),
			NotificationManager.IMPORTANCE_LOW,
		).apply {
			lightColor = Color.BLUE
			description = context.getString(R.string.channel_description)
			lockscreenVisibility = NotificationCompat.VISIBILITY_PRIVATE
		}

		context.getSystemService(NotificationManager::class.java)
			?.createNotificationChannel(channel)
	}

	fun buildVpnNotification(state: Tunnel.State, entry: EntryPoint?, exit: ExitPoint?, gatewaysEntry: List<NymGateway>?, gatewaysExit: List<NymGateway>?): Notification {
		setupChannel()

		val title = context.getString(R.string.vpn_notification_title)
		val stateText = state.toStateText()

		val entryText = entry?.let {
			context.getString(R.string.notification_entry, formatEntry(it, gatewaysEntry))
		}

		val exitText = exit?.let {
			context.getString(R.string.notification_exit, formatExit(it, gatewaysExit))
		}

		val fullText = buildList {
			add(stateText)
			entryText?.let(::add)
			exitText?.let(::add)
		}.joinToString("\n")

		val stopIntent = Intent(context, StopVpnReceiver::class.java).apply {
			action = StopVpnReceiver.ACTION_DISCONNECT
		}

		val stopPendingIntent = PendingIntent.getBroadcast(
			context,
			0,
			stopIntent,
			PendingIntent.FLAG_IMMUTABLE,
		)

		return NotificationCompat.Builder(context, VPN_CHANNEL_ID)
			.setOngoing(true)
			.setContentTitle(title)
			.setContentText(stateText)
			.setStyle(NotificationCompat.BigTextStyle().bigText(fullText))
			.setSmallIcon(R.drawable.ic_stat_name)
			.setContentIntent(contentIntent())
			.addAction(R.drawable.ic_stop, context.getString(R.string.disconnect), stopPendingIntent)
			.setCategory(Notification.CATEGORY_SERVICE)
			.build()
	}

	/** Minimal notification used when promoting to foreground early. */
	fun buildMinimalNotification(): Notification {
		setupChannel()

		val title = context.getString(R.string.vpn_notification_title)

		return NotificationCompat.Builder(context, VPN_CHANNEL_ID)
			.setOngoing(true)
			.setContentTitle(title)
			.setSmallIcon(R.drawable.ic_stat_name)
			.setCategory(Notification.CATEGORY_SERVICE)
			.build()
	}

	/** Updates/cancels the foreground notification based on [state]. */
	internal fun updateVpnNotification(state: Tunnel.State, entry: EntryPoint?, exit: ExitPoint?, gatewaysEntry: List<NymGateway>?, gatewaysExit: List<NymGateway>?) {
		withNotificationPermission {
			val nm = NotificationManagerCompat.from(context)
			if (state == Tunnel.State.Down) {
				nm.cancel(VPN_FOREGROUND_ID)
			} else {
				nm.notify(
					VPN_FOREGROUND_ID,
					buildVpnNotification(state, entry, exit, gatewaysEntry, gatewaysExit),
				)
			}
		}
	}

	private fun contentIntent(): PendingIntent {
		val intent = Intent().apply {
			setClassName(context.packageName, "net.nymtech.nymvpn.ui.MainActivity")
			flags = Intent.FLAG_ACTIVITY_CLEAR_TOP or Intent.FLAG_ACTIVITY_SINGLE_TOP
			action = Intent.ACTION_MAIN
		}

		return PendingIntent.getActivity(context, 0, intent, pendingIntentFlags)
	}

	private val pendingIntentFlags: Int by lazy {
		when {
			Build.VERSION.SDK_INT >= Build.VERSION_CODES.UPSIDE_DOWN_CAKE ->
				PendingIntent.FLAG_UPDATE_CURRENT or
					PendingIntent.FLAG_MUTABLE or
					PendingIntent.FLAG_ALLOW_UNSAFE_IMPLICIT_INTENT

			Build.VERSION.SDK_INT >= Build.VERSION_CODES.S ->
				PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_MUTABLE

			else -> PendingIntent.FLAG_UPDATE_CURRENT
		}
	}

	private fun formatEntry(entry: EntryPoint, gateways: List<NymGateway>?): String {
		val (name, city, countryIso) = when (entry) {
			is EntryPoint.Gateway -> {
				val gw = gateways?.firstOrNull { it.identity == entry.identity }
				Triple(gw?.name ?: entry.identity, gw?.city, gw?.twoLetterCountryISO)
			}

			is EntryPoint.Region -> {
				val gw = gateways?.firstOrNull { it.region.equals(entry.region, ignoreCase = true) }
				Triple(gw?.region ?: entry.region, gw?.city, gw?.twoLetterCountryISO)
			}

			is EntryPoint.Country -> Triple(toDisplayCountry(entry.twoLetterIsoCountryCode), null, null)
			is EntryPoint.Random -> Triple(GatewaySelectionMode.RANDOM.value, null, null)
			is EntryPoint.Auto -> Triple(GatewaySelectionMode.AUTO.value, null, null)
		}

		return formatNodeLocation(name, city, countryIso)
	}

	private fun formatExit(exit: ExitPoint, gateways: List<NymGateway>?): String {
		val (name, city, countryIso) = when (exit) {
			is ExitPoint.Gateway -> {
				val gw = gateways?.firstOrNull { it.identity == exit.identity }
				Triple(gw?.name ?: exit.identity, gw?.city, gw?.twoLetterCountryISO)
			}

			is ExitPoint.Region -> {
				val gw = gateways?.firstOrNull { it.region.equals(exit.region, ignoreCase = true) }
				Triple(gw?.region ?: exit.region, gw?.city, gw?.twoLetterCountryISO)
			}

			is ExitPoint.Country -> Triple(toDisplayCountry(exit.twoLetterIsoCountryCode), null, null)
			is ExitPoint.Address -> Triple(exit.address, null, null)
			is ExitPoint.Random -> Triple(GatewaySelectionMode.RANDOM.value.lowercase(), null, null)
			is ExitPoint.Auto -> Triple(GatewaySelectionMode.AUTO.value.lowercase(), null, null)
		}

		return formatNodeLocation(name, city, countryIso)
	}

	private fun Tunnel.State.toStateText(): String = when (this) {
		Tunnel.State.Down -> context.getString(R.string.state_disconnected)
		Tunnel.State.Up -> context.getString(R.string.state_connected)
		Tunnel.State.InitializingClient -> context.getString(R.string.state_initializing)
		Tunnel.State.EstablishingConnection -> context.getString(R.string.state_establishing)
		is Tunnel.State.Error -> this.reason.toHumanReadableString(context)
		else -> toString()
	}
}

private fun formatNodeLocation(nodeName: String, city: String?, countryIso: String?): String {
	val country = countryIso?.let { toDisplayCountry(it) }
	return when {
		city != null && country != null -> "$nodeName ($city, $country)"
		country != null -> "$nodeName ($country)"
		else -> nodeName
	}
}
