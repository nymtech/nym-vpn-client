package net.nymtech.vpn

import android.app.Service
import android.content.Context
import android.content.Intent
import android.os.Build
import timber.log.Timber


object ServiceManager {

  private fun <T : Service> actionOnService(
      action: Action,
      context: Context,
      cls: Class<T>,
      extras: Map<String, String>? = null
  ) {
    val intent =
        Intent(context, cls).also {
          it.action = action.name
          extras?.forEach { (k, v) -> it.putExtra(k, v) }
        }
    intent.component?.javaClass
    try {
      when (action) {
        Action.START_FOREGROUND -> {
          if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
            context.startForegroundService(intent)
          } else context.startService(intent)
        }
        Action.START -> {
            Timber.d("Start intent")
          context.startService(intent)
        }
        Action.STOP -> context.startService(intent)
      }
    } catch (e: Exception) {
      Timber.e(e.message)
    }
  }

    fun startVpnService(context: Context) {
        Timber.d("Called start vpn service")
        actionOnService(
            Action.START,
            context,
            NymVpnService::class.java,
        )
    }

    fun stopVpnService(context: Context) {
        actionOnService(
            Action.STOP,
            context,
            NymVpnService::class.java,
        )
    }
}
