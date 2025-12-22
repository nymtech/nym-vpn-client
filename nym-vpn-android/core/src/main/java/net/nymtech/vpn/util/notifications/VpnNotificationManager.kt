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
import net.nymtech.vpn.util.extensions.toDisplayCountry
import nym_vpn_lib_types.EntryPoint
import nym_vpn_lib_types.ExitPoint

@SuppressLint("MissingPermission")
internal class VpnNotificationManager private constructor(private val context: Context) {

	companion object : SingletonHolder<VpnNotificationManager, Context>(::VpnNotificationManager) {
		const val VPN_CHANNEL_ID = "VpnForegroundChannel"
		const val VPN_FOREGROUND_ID = 223
	}

	inline fun withNotificationPermission(action: () -> Unit) {
		if (ActivityCompat.checkSelfPermission(
				context,
				Manifest.permission.POST_NOTIFICATIONS,
			) == PackageManager.PERMISSION_GRANTED
		) {
			action()
		}
	}

	private fun setupChannel() {
		if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
			val channel = NotificationChannel(
				VPN_CHANNEL_ID,
				context.getString(R.string.channel_name),
				NotificationManager.IMPORTANCE_LOW,
			).apply {
				lightColor = Color.BLUE
				description = context.getString(R.string.channel_description)
				lockscreenVisibility = NotificationCompat.VISIBILITY_PRIVATE
			}
			context.getSystemService(NotificationManager::class.java)?.createNotificationChannel(channel)
		}
	}

	fun buildVpnNotification(state: Tunnel.State, entry: EntryPoint?, exit: ExitPoint?, gatewaysEntry: List<NymGateway>?, gatewaysExit: List<NymGateway>?): Notification {
		setupChannel()

		val title = context.getString(R.string.vpn_notification_title)
		val entryText = entry?.let { e ->
			val (nodeName, city, countryIso) = when (e) {
				is EntryPoint.Gateway -> {
					val gw = gatewaysEntry?.firstOrNull { it.identity == e.identity }
					Triple(
						gw?.name ?: e.identity,
						gw?.city,
						gw?.twoLetterCountryISO,
					)
				}

				is EntryPoint.Region -> {
					val gw = gatewaysEntry?.firstOrNull { it.region.equals(e.region, ignoreCase = true) }
					Triple(
						gw?.region ?: e.region,
						gw?.twoLetterCountryISO,
						null,
					)
				}

				is EntryPoint.Country -> Triple(
					toDisplayCountry(e.twoLetterIsoCountryCode),
					null,
					null,
				)

				is EntryPoint.Random -> Triple("random", null, null)
			}

			val formatted = formatNodeLocation(nodeName, city, countryIso)
			context.getString(R.string.notification_entry, formatted)
		}

		val exitText = exit?.let { e ->
			val (nodeName, city, countryIso) = when (e) {
				is ExitPoint.Gateway -> {
					val gw = gatewaysExit?.firstOrNull { it.identity == e.identity }
					Triple(
						gw?.name ?: e.identity,
						gw?.city,
						gw?.twoLetterCountryISO,
					)
				}

				is ExitPoint.Region -> {
					val gw = gatewaysExit?.firstOrNull { it.region.equals(e.region, ignoreCase = true) }
					Triple(
						gw?.region ?: e.region,
						gw?.twoLetterCountryISO,
						null,
					)
				}

				is ExitPoint.Country -> Triple(
					toDisplayCountry(e.twoLetterIsoCountryCode),
					null,
					null,
				)

				is ExitPoint.Address -> Triple(e.address, null, null)

				is ExitPoint.Random -> Triple("random", null, null)
			}

			val formatted = formatNodeLocation(nodeName, city, countryIso)
			context.getString(R.string.notification_exit, formatted)
		}

		val text = when (state) {
			Tunnel.State.Down -> context.getString(R.string.state_disconnected)
			Tunnel.State.Up -> context.getString(R.string.state_connected)
			Tunnel.State.InitializingClient -> context.getString(R.string.state_initializing)
			Tunnel.State.EstablishingConnection -> context.getString(R.string.state_establishing)
			else -> state.toString()
		}

		val lines = buildList {
			add(text)
			entryText?.let { add(it) }
			exitText?.let { add(it) }
		}
		val fullText = lines.joinToString("\n")

		val stopIntent = Intent(context, StopVpnReceiver::class.java)
		val pending = PendingIntent.getBroadcast(
			context,
			0,
			stopIntent,
			PendingIntent.FLAG_IMMUTABLE,
		)

		return NotificationCompat.Builder(context, VPN_CHANNEL_ID)
			.setOngoing(true)
			.setContentTitle(title)
			.setContentText(text)
			.setStyle(NotificationCompat.BigTextStyle().bigText(fullText))
			.setSmallIcon(R.drawable.ic_stat_name)
			.setContentIntent(contentIntent())
			.addAction(R.drawable.ic_stop, context.getString(R.string.disconnect), pending)
			.setCategory(Notification.CATEGORY_SERVICE)
			.build()
	}

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

	internal fun updateVpnNotification(state: Tunnel.State, entry: EntryPoint?, exit: ExitPoint?, gatewaysEntry: List<NymGateway>?, gatewaysExit: List<NymGateway>?) {
		withNotificationPermission {
			val notificationManager = NotificationManagerCompat.from(context)
			if (state == Tunnel.State.Down) {
				notificationManager.cancel(VPN_FOREGROUND_ID)
			} else {
				notificationManager.notify(
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
			Build.VERSION.SDK_INT >= Build.VERSION_CODES.UPSIDE_DOWN_CAKE -> {
				PendingIntent.FLAG_UPDATE_CURRENT or
					PendingIntent.FLAG_MUTABLE or
					PendingIntent.FLAG_ALLOW_UNSAFE_IMPLICIT_INTENT
			}

			Build.VERSION.SDK_INT >= Build.VERSION_CODES.S -> {
				PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_MUTABLE
			}

			else -> PendingIntent.FLAG_UPDATE_CURRENT
		}
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
