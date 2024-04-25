package net.nymtech.nymvpn.receiver

import android.app.AlarmManager
import android.app.AlarmManager.ACTION_SCHEDULE_EXACT_ALARM_PERMISSION_STATE_CHANGED
import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent
import android.os.Build
import androidx.annotation.RequiresApi
import timber.log.Timber

class AlarmPermissionReceiver : BroadcastReceiver() {

	// TODO this is not working yet, perhaps not needed
	@RequiresApi(Build.VERSION_CODES.S)
	override fun onReceive(context: Context, intent: Intent) {
		val alarmManager: AlarmManager = context.getSystemService(Context.ALARM_SERVICE) as AlarmManager
		when (intent.action) {
			ACTION_SCHEDULE_EXACT_ALARM_PERMISSION_STATE_CHANGED -> {
				if (alarmManager.canScheduleExactAlarms()) {
					Timber.d("Schedule exact alarms granted")
				} else {
					Timber.w("Exact alarms permission removed")
				}
			}
		}
	}
}
