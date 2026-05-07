package net.nymtech.nymvpn.ui.common.snackbar

import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Cancel
import androidx.compose.material.icons.filled.CheckCircle
import androidx.compose.material.icons.filled.Warning
import androidx.compose.material.icons.outlined.Info
import androidx.compose.ui.graphics.vector.ImageVector
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow

enum class AlertType { Confirmation, Neutral, Negative, Warning }

data class NymAlertMessage(
	val type: AlertType = AlertType.Neutral,
	val title: String,
	val body: String? = null,
	val action: NymAlertAction? = null,
	val duration: Long = 4_000L,
	val onDismiss: (() -> Unit)? = null,
)

data class NymAlertAction(val label: String, val onClick: () -> Unit)

object NymAlertController {
	private val _message = MutableStateFlow<NymAlertMessage?>(null)
	val message: StateFlow<NymAlertMessage?> = _message.asStateFlow()

	fun show(message: NymAlertMessage) {
		_message.value = message
	}

	fun dismiss() {
		val msg = _message.value
		_message.value = null
		msg?.onDismiss?.invoke()
	}
}

internal val AlertType.icon: ImageVector
	get() = when (this) {
		AlertType.Confirmation -> Icons.Filled.CheckCircle
		AlertType.Neutral -> Icons.Outlined.Info
		AlertType.Negative -> Icons.Filled.Cancel
		AlertType.Warning -> Icons.Filled.Warning
	}
