package net.nymtech.vpn.util.notifications

import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent
import android.widget.Toast
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.DelicateCoroutinesApi
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.GlobalScope
import kotlinx.coroutines.launch
import net.nymtech.vpn.backend.NymBackend
import kotlin.coroutines.CoroutineContext
import kotlin.coroutines.EmptyCoroutineContext

class StopVpnReceiver : BroadcastReceiver() {

	override fun onReceive(context: Context, intent: Intent?) = goAsync { pendingResult ->
		val backend = NymBackend.instance as? NymBackend
		if (backend == null) {
			Toast.makeText(context, "VPN not running", Toast.LENGTH_SHORT).show()
			return@goAsync
		}
		backend.stop()
		GlobalScope.launch(Dispatchers.Main) {
			Toast.makeText(context, "VPN disconnected", Toast.LENGTH_SHORT).show()
		}
	}
}

fun BroadcastReceiver.goAsync(context: CoroutineContext = EmptyCoroutineContext, block: suspend CoroutineScope.(BroadcastReceiver.PendingResult) -> Unit) {
	val pendingResult = goAsync()
	@OptIn(DelicateCoroutinesApi::class)
	GlobalScope.launch(context) {
		try {
			block(pendingResult)
		} finally {
			pendingResult.finish()
		}
	}
}
