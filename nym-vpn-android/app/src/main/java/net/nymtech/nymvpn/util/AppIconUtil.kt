package net.nymtech.nymvpn.util

import android.content.ComponentName
import android.content.Context
import android.content.Intent
import android.content.pm.PackageManager
import net.nymtech.nymvpn.data.domain.AppIcon
import timber.log.Timber
import kotlin.system.exitProcess

object AppIconUtil {

	private const val TAG = "app-icon"

	fun getCurrent(context: Context): AppIcon {
		val pm = context.packageManager
		AppIcon.entries.forEach { icon ->
			val state = pm.getComponentEnabledSetting(ComponentName(context, icon.componentName))
			val active = state == PackageManager.COMPONENT_ENABLED_STATE_ENABLED ||
				(state == PackageManager.COMPONENT_ENABLED_STATE_DEFAULT && icon == AppIcon.DEFAULT)
			if (active) return icon
		}
		return AppIcon.DEFAULT
	}

	fun apply(context: Context, target: AppIcon) {
		switchActiveAlias(context, target)
		relaunch(context, target.componentName)
	}

	internal fun switchActiveAlias(context: Context, target: AppIcon) {
		val pm = context.packageManager
		AppIcon.entries.forEach { icon ->
			val newState = if (icon == target) {
				PackageManager.COMPONENT_ENABLED_STATE_ENABLED
			} else {
				PackageManager.COMPONENT_ENABLED_STATE_DISABLED
			}
			pm.setComponentEnabledSetting(
				ComponentName(context, icon.componentName),
				newState,
				PackageManager.DONT_KILL_APP,
			)
		}
		Timber.tag(TAG).d("Switched app icon to %s", target.name)
	}

	private fun relaunch(context: Context, componentName: String) {
		val intent = Intent().apply {
			component = ComponentName(context, componentName)
			addFlags(Intent.FLAG_ACTIVITY_NEW_TASK or Intent.FLAG_ACTIVITY_CLEAR_TASK)
		}
		context.startActivity(intent)
		exitProcess(0)
	}
}
