package net.nymtech.nymvpn.ui.common.snackbar

import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.core.tween
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.slideInVertically
import androidx.compose.animation.slideOutVertically
import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Close
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import net.nymtech.nymvpn.R
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import kotlinx.coroutines.delay
import net.nymtech.nymvpn.ui.theme.LocalNymColors

@Composable
fun NymAlertHost(modifier: Modifier = Modifier, previewMessage: NymAlertMessage? = null) {
	val controllerMessage by NymAlertController.message.collectAsStateWithLifecycle()
	val message = previewMessage ?: controllerMessage

	var lastMessage by remember { mutableStateOf<NymAlertMessage?>(null) }
	if (message != null) lastMessage = message

	AnimatedVisibility(
		visible = message != null,
		enter = slideInVertically(tween(300)) { -it } + fadeIn(tween(300)),
		exit = slideOutVertically(tween(250)) { -it } + fadeOut(tween(250)),
		modifier = modifier,
	) {
		lastMessage?.let { msg ->
			NymAlert(message = msg, onDismiss = { NymAlertController.dismiss() })
		}
	}

	LaunchedEffect(message) {
		val msg = message ?: return@LaunchedEffect
		if (msg.duration < Long.MAX_VALUE) {
			delay(msg.duration)
			NymAlertController.dismiss()
		}
	}
}

@Composable
private fun NymAlert(message: NymAlertMessage, onDismiss: () -> Unit, modifier: Modifier = Modifier) {
	val isError = message.type == AlertType.Error
	val background = if (isError) MaterialTheme.colorScheme.errorContainer else MaterialTheme.colorScheme.inverseSurface
	val onAlertSurface = if (isError) MaterialTheme.colorScheme.onError else MaterialTheme.colorScheme.inverseOnSurface
	val iconTint = when (message.type) {
		AlertType.Confirmation -> MaterialTheme.colorScheme.primary
		AlertType.Neutral -> onAlertSurface
		AlertType.Negative -> MaterialTheme.colorScheme.error
		AlertType.Warning -> LocalNymColors.current.warning
		AlertType.Error -> onAlertSurface
	}

	Surface(
		modifier = modifier.fillMaxWidth(),
		shape = RoundedCornerShape(12.dp),
		color = background,
		shadowElevation = 6.dp,
	) {
		Column(
			modifier = Modifier
				.fillMaxWidth()
				.padding(16.dp),
			verticalArrangement = Arrangement.spacedBy(if (isError) 12.dp else 5.dp),
		) {
			Row(
				modifier = Modifier.fillMaxWidth(),
				horizontalArrangement = Arrangement.spacedBy(16.dp),
				verticalAlignment = Alignment.Top,
			) {
				Icon(
					imageVector = message.type.icon,
					contentDescription = null,
					tint = iconTint,
					modifier = Modifier.size(24.dp),
				)
				Column(
					modifier = Modifier.weight(1f),
					verticalArrangement = Arrangement.spacedBy(2.dp),
				) {
					Text(
						text = message.title,
						style = MaterialTheme.typography.titleSmall,
						color = onAlertSurface,
					)
					message.body?.let { body ->
						Text(
							text = body,
							style = MaterialTheme.typography.bodySmall,
							color = onAlertSurface,
						)
					}
				}
				if (!isError) {
					IconButton(onClick = onDismiss) {
						Icon(
							imageVector = Icons.Filled.Close,
							contentDescription = stringResource(R.string.close),
							tint = onAlertSurface,
							modifier = Modifier.size(24.dp),
						)
					}
				}
			}
			message.action?.let { action ->
				Row(
					modifier = Modifier.fillMaxWidth(),
					horizontalArrangement = Arrangement.End,
				) {
					OutlinedButton(
						onClick = {
							action.onClick()
							onDismiss()
						},
						shape = CircleShape,
						border = BorderStroke(1.5.dp, onAlertSurface),
						contentPadding = PaddingValues(horizontal = 12.dp, vertical = 8.dp),
					) {
						Text(
							text = action.label,
							style = MaterialTheme.typography.labelMedium.copy(fontWeight = FontWeight.Bold),
							color = onAlertSurface,
						)
					}
				}
			}
		}
	}
}
