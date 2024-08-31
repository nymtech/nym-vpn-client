package net.nymtech.vpn.util

import android.Manifest
import android.app.Notification
import android.app.NotificationChannel
import android.app.NotificationManager
import android.content.Context
import android.content.pm.PackageManager
import android.graphics.Color
import android.net.VpnService
import android.os.Build
import androidx.core.app.ActivityCompat
import androidx.core.app.NotificationCompat
import androidx.core.app.NotificationManagerCompat
import net.nymtech.vpn.R

internal object NotificationManager {

	const val VPN_NOTIFICATION_ID = 222
	private const val VPN_CHANNEL_ID = "vpnChannel"

	fun createNotificationChannel(context: Context) {
		if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
			// Create the NotificationChannel.
			val name = context.getString(R.string.channel_name)
			val descriptionText = context.getString(R.string.channel_description)
			val importance = NotificationManager.IMPORTANCE_HIGH
			val mChannel = NotificationChannel(VPN_CHANNEL_ID, name, importance).apply {
				lightColor = Color.BLUE
				lockscreenVisibility = Notification.VISIBILITY_PRIVATE
			}
			mChannel.description = descriptionText
			// Register the channel with the system. You can't change the importance
			// or other notification behaviors after this.
			val notificationManager = context.getSystemService(VpnService.NOTIFICATION_SERVICE) as NotificationManager
			notificationManager.createNotificationChannel(mChannel)
		}
	}

	fun createVpnRunningNotification(context: Context): Notification {
		val notificationBuilder = NotificationCompat.Builder(context, VPN_CHANNEL_ID)
		return notificationBuilder.setOngoing(true)
			.setContentTitle(context.getString(R.string.vpn_notification_title))
			.setContentText(context.getString(R.string.vpn_notification_text))
			.setSmallIcon(R.drawable.ic_stat_name)
			.setCategory(Notification.CATEGORY_SERVICE)
			.build()
	}

	fun createVpnFailedNotification(context: Context): Notification {
		val notificationBuilder = NotificationCompat.Builder(context, VPN_CHANNEL_ID)
		return notificationBuilder.setOngoing(false)
			.setContentTitle(context.getString(R.string.vpn_notification_title))
			.setContentText(context.getString(R.string.failed))
			.setSmallIcon(R.drawable.ic_stat_name)
			.setCategory(Notification.CATEGORY_SERVICE)
			.build()
	}

	fun notify(context: Context, notification: Notification) {
		with(NotificationManagerCompat.from(context)) {
			if (ActivityCompat.checkSelfPermission(
					context,
					Manifest.permission.POST_NOTIFICATIONS,
				) == PackageManager.PERMISSION_GRANTED
			) {
				notify(VPN_NOTIFICATION_ID, notification)
			}
		}
	}
}
