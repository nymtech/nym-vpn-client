package net.nymtech.nymvpn.util

import android.content.ComponentName
import android.content.Context
import android.content.Intent
import android.content.pm.PackageManager
import net.nymtech.nymvpn.data.domain.AppIcon
import timber.log.Timber
import kotlin.system.exitProcess

object AppIconUtil {
    /**
     * Read the active launcher icon from the system. On a fresh install all
     * aliases are in COMPONENT_ENABLED_STATE_DEFAULT and the manifest's
     * `android:enabled` attribute is the truth — so DEFAULT counts as enabled
     * for whichever icon the manifest marks default.
     */
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

    /**
     * Switch the launcher icon. The app process is restarted so the system
     * picks up the new alias cleanly. Caller MUST ensure the VPN tunnel is
     * disconnected first — exitProcess kills any in-process VPN service.
     */
    fun apply(context: Context, target: AppIcon) {
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
        Timber.d("AppIconUtil: switched to %s", target.name)
        relaunch(context, target.componentName)
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
