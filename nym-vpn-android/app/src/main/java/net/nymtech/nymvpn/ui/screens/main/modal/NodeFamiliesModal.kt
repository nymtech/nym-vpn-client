package net.nymtech.nymvpn.ui.screens.main.modal

import androidx.compose.foundation.clickable
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.runtime.remember
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Warning
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.SpanStyle
import androidx.compose.ui.text.buildAnnotatedString
import androidx.compose.ui.text.font.Font
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.style.TextDecoration
import androidx.compose.ui.text.withStyle
import androidx.compose.ui.unit.dp
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.common.Modal
import net.nymtech.nymvpn.ui.common.buttons.MainStyledButton
import net.nymtech.nymvpn.ui.common.buttons.TransparentButton
import net.nymtech.nymvpn.ui.theme.CustomTypography
import net.nymtech.nymvpn.util.extensions.scaledHeight

@Composable
fun NodeFamiliesModal(showDialog: Boolean, onConfirmClick: () -> Unit, onNotificationSettingsClick: () -> Unit, onDismiss: () -> Unit) {
	Modal(
		show = showDialog,
		onDismiss = onDismiss,
		icon = Icons.Filled.Warning,
		iconTint = MaterialTheme.colorScheme.primary,
		title = {
			Text(
				text = stringResource(R.string.node_families_title),
				style = CustomTypography.labelHuge,
				color = MaterialTheme.colorScheme.onPrimaryContainer,
				fontFamily = FontFamily(Font(R.font.lab_grotesque_regular)),
			)
		},
		text = {
			NodeFamiliesDescriptionText(onNotificationSettingsClick)
		},
		confirmButton = {
			MainStyledButton(
				onClick = onConfirmClick,
				textColor = Color.Black,
				content = {
					Text(
						stringResource(R.string.node_families_connect_button),
						fontFamily = FontFamily(Font(R.font.lab_grotesque_regular)),
					)
				},
				modifier = Modifier
					.fillMaxWidth()
					.height(40.dp.scaledHeight()),
			)
		},
		dismissButton = {
			TransparentButton(
				onClick = onDismiss,
				content = {
					Text(
						stringResource(R.string.cancel),
						fontFamily = FontFamily(Font(R.font.lab_grotesque_regular)),
						color = MaterialTheme.colorScheme.onPrimaryContainer,
					)
				},
				modifier = Modifier
					.fillMaxWidth()
					.height(40.dp.scaledHeight()),
			)
		},
	)
}

@Composable
private fun NodeFamiliesDescriptionText(onLinkClick: () -> Unit) {
	val annotatedString = buildAnnotatedString {
		append(stringResource(R.string.node_families_description_text))
		append(" ")
		withStyle(
			SpanStyle(
				textDecoration = TextDecoration.Underline,
			),
		) {
			append(stringResource(R.string.node_families_description_link))
		}
	}
	Text(
		text = annotatedString,
		style = MaterialTheme.typography.bodyMedium,
		color = MaterialTheme.colorScheme.onPrimaryContainer,
		modifier = Modifier.clickable(
			indication = null,
			interactionSource = remember { MutableInteractionSource() },
		) { onLinkClick() },
	)
}
