package net.nymtech.nymvpn.service.notification

import android.app.NotificationManager
import android.app.PendingIntent
import androidx.core.app.NotificationCompat

interface NotificationService {

	val channelName: String
	val channelDescription: String
	val builder: NotificationCompat.Builder

	fun showNotification(
		title: String,
		action: PendingIntent? = null,
		actionText: String? = null,
		description: String,
		showTimestamp: Boolean = false,
		importance: Int = NotificationManager.IMPORTANCE_HIGH,
		vibration: Boolean = false,
		onGoing: Boolean = false,
		lights: Boolean = true,
		onlyAlertOnce: Boolean = true,
	)
}
