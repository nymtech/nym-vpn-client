package net.nymtech.vpn

import android.app.Notification
import android.app.NotificationChannel
import android.app.NotificationManager
import android.content.Context
import android.graphics.Color
import android.net.VpnService
import android.os.Build
import androidx.core.app.NotificationCompat
import net.nymtech.vpn_client.R

internal object NotificationService {

	const val VPN_CHANNEL_ID = "vpnChannel"
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
}
