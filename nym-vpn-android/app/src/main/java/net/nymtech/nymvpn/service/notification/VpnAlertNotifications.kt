package net.nymtech.nymvpn.service.notification

import android.Manifest
import android.app.NotificationChannel
import android.app.PendingIntent
import android.content.Context
import android.content.Intent
import android.content.pm.PackageManager
import android.graphics.Color
import android.os.Build
import androidx.annotation.RequiresApi
import androidx.core.app.ActivityCompat
import androidx.core.app.NotificationCompat
import androidx.core.app.NotificationManagerCompat
import dagger.hilt.android.qualifiers.ApplicationContext
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.MainActivity
import javax.inject.Inject

class VpnAlertNotifications @Inject constructor(@ApplicationContext private val context: Context) : NotificationService {

	override val channelName: String = context.getString(R.string.vpn_alerts_channel_id)
	override val channelDescription: String = context.getString(R.string.vpn_alerts_channel_description)
	override val builder: NotificationCompat.Builder
		get() = NotificationCompat.Builder(context, ALERTS_CHANNEL_ID)

	override fun showNotification(
		title: String,
		action: PendingIntent?,
		actionText: String?,
		description: String,
		showTimestamp: Boolean,
		importance: Int,
		vibration: Boolean,
		onGoing: Boolean,
		lights: Boolean,
		onlyAlertOnce: Boolean,
	) {
		val notificationManager = NotificationManagerCompat.from(context)

		if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
			val legacyChannel = if (notificationManager.getNotificationChannel(ALERTS_CHANNEL_ID) == null) {
				legacyChannels(notificationManager).let { legacy -> legacy.firstOrNull { it.id == channelName } ?: legacy.firstOrNull() }
			} else {
				null
			}
			deleteLegacyChannels(notificationManager)
			// Stable id; the localized string is only the display name (was used as id per locale).
			val channel = NotificationChannel(
				ALERTS_CHANNEL_ID,
				channelName,
				legacyChannel?.importance ?: importance,
			).apply {
				this.description = channelDescription
				enableLights(legacyChannel?.shouldShowLights() ?: lights)
				lightColor = Color.RED
				enableVibration(legacyChannel?.shouldVibrate() ?: vibration)
				vibrationPattern = longArrayOf(100, 200, 300)
				legacyChannel?.let {
					setSound(it.sound, it.audioAttributes)
					lockscreenVisibility = it.lockscreenVisibility
					setShowBadge(it.canShowBadge())
				}
			}
			notificationManager.createNotificationChannel(channel)
		}

		val contentPendingIntent: PendingIntent =
			Intent(context, MainActivity::class.java).let { notificationIntent ->
				PendingIntent.getActivity(
					context,
					0,
					notificationIntent,
					PendingIntent.FLAG_IMMUTABLE,
				)
			}

		val b = builder
			.setContentTitle(title)
			.setContentText(description)
			.setOnlyAlertOnce(onlyAlertOnce)
			.setContentIntent(contentPendingIntent)
			.setOngoing(onGoing)
			.setPriority(NotificationCompat.PRIORITY_HIGH)
			.setShowWhen(showTimestamp)
			.setSmallIcon(net.nymtech.vpn.R.drawable.ic_stat_name)

		if (action != null && actionText != null) {
			b.addAction(NotificationCompat.Action.Builder(0, actionText, action).build())
			b.setAutoCancel(true)
		}

		if (ActivityCompat.checkSelfPermission(
				context,
				Manifest.permission.POST_NOTIFICATIONS,
			) == PackageManager.PERMISSION_GRANTED
		) {
			notificationManager.notify(NOTIFICATION_ID, b.build())
		}
	}

	override fun clearNotifications() {
		val notificationManager = NotificationManagerCompat.from(context)
		if (ActivityCompat.checkSelfPermission(
				context,
				Manifest.permission.POST_NOTIFICATIONS,
			) == PackageManager.PERMISSION_GRANTED
		) {
			notificationManager.cancel(NOTIFICATION_ID)
		}
	}

	// Legacy alert channels used the localized name as id (id == name); remove those orphans.
	@RequiresApi(Build.VERSION_CODES.O)
	private fun legacyChannels(notificationManager: NotificationManagerCompat): List<NotificationChannel> =
		notificationManager.notificationChannels
			.filter { it.id != ALERTS_CHANNEL_ID && it.id == it.name?.toString() }

	@RequiresApi(Build.VERSION_CODES.O)
	private fun deleteLegacyChannels(notificationManager: NotificationManagerCompat) {
		legacyChannels(notificationManager)
			.forEach { runCatching { notificationManager.deleteNotificationChannel(it.id) } }
	}

	companion object {
		private const val NOTIFICATION_ID = 42
		private const val ALERTS_CHANNEL_ID = "vpn_alerts"
	}
}
