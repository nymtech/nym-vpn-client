package net.nymtech.nymvpn.service.notification

import android.Manifest
import android.app.NotificationChannel
import android.app.PendingIntent
import android.content.Context
import android.content.Intent
import android.content.pm.PackageManager
import android.graphics.Color
import android.os.Build
import androidx.core.app.ActivityCompat
import androidx.core.app.NotificationCompat
import androidx.core.app.NotificationManagerCompat
import dagger.hilt.android.qualifiers.ApplicationContext
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.MainActivity
import javax.inject.Inject

class VpnAlertNotifications
@Inject
constructor(
	@ApplicationContext
	private val context: Context,
) :
	NotificationService {

	override val channelName: String = context.getString(R.string.vpn_alerts_channel_id)
	override val channelDescription: String = context.getString(R.string.vpn_alerts_channel_description)

	override val builder: NotificationCompat.Builder =
		NotificationCompat.Builder(
			context,
			channelName,
		)

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
		// Create notification channel for Android Oreo and above
		if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
			val channel = NotificationChannel(
				channelName,
				channelName,
				importance,
			)
				.let {
					it.description = title
					it.enableLights(lights)
					it.lightColor = Color.RED
					it.enableVibration(vibration)
					it.vibrationPattern = longArrayOf(100, 200, 300)
					it
				}
			notificationManager.createNotificationChannel(channel)
		} else {
			Unit
		}
		val pendingIntent: PendingIntent =
			Intent(context, MainActivity::class.java).let { notificationIntent ->
				PendingIntent.getActivity(
					context,
					0,
					notificationIntent,
					PendingIntent.FLAG_IMMUTABLE,
				)
			}

		val notification = builder.let {
			if (action != null && actionText != null) {
				it.addAction(
					NotificationCompat.Action.Builder(0, actionText, action).build(),
				)
				it.setAutoCancel(true)
			}
			it.setContentTitle(title)
				.setContentText(description)
				.setOnlyAlertOnce(onlyAlertOnce)
				.setContentIntent(pendingIntent)
				.setOngoing(onGoing)
				.setPriority(NotificationCompat.PRIORITY_HIGH)
				.setShowWhen(showTimestamp)
				.setSmallIcon(net.nymtech.vpn.R.drawable.ic_stat_name)
				.build()
		}
		with(notificationManager) {
			if (ActivityCompat.checkSelfPermission(
					context,
					Manifest.permission.POST_NOTIFICATIONS,
				) == PackageManager.PERMISSION_GRANTED
			) {
				notify(NOTIFICATION_ID, notification)
			}
		}
	}
	companion object {
		private const val NOTIFICATION_ID = 42
	}
}
