package net.nymtech.nymvpn.ui.common.snackbar

import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Cancel
import androidx.compose.material.icons.filled.CheckCircle
import androidx.compose.material.icons.filled.Error
import androidx.compose.material.icons.filled.FmdBad
import androidx.compose.material.icons.filled.Info
import androidx.compose.ui.graphics.vector.ImageVector
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow

enum class AlertType { Confirmation, Neutral, Negative, Warning, Error }

enum class AlertId { PendingSubscription, ExpiryWarning, Expired, ConnectionError }

data class AlertMessage(
	val type: AlertType = AlertType.Neutral,
	val title: String,
	val body: String? = null,
	val action: AlertAction? = null,
	val duration: Long = 4_000L,
	val onDismiss: (() -> Unit)? = null,
	val id: AlertId? = null,
)

data class AlertAction(val label: String, val onClick: () -> Unit)

object AlertController {
	private val _message = MutableStateFlow<AlertMessage?>(null)
	val message: StateFlow<AlertMessage?> = _message.asStateFlow()

	fun show(message: AlertMessage) {
		_message.value = message
	}

	fun dismiss(id: AlertId? = null) {
		if (id != null && _message.value?.id != id) return
		val msg = _message.value
		_message.value = null
		msg?.onDismiss?.invoke()
	}
}

internal val AlertType.icon: ImageVector
	get() = when (this) {
		AlertType.Confirmation -> Icons.Filled.CheckCircle
		AlertType.Neutral -> Icons.Filled.Info
		AlertType.Negative -> Icons.Filled.Cancel
		AlertType.Warning -> Icons.Filled.FmdBad
		AlertType.Error -> Icons.Filled.Error
	}
