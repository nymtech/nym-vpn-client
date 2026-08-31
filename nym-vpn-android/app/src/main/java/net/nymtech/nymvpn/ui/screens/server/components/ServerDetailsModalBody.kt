package net.nymtech.nymvpn.ui.screens.server.components

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.outlined.OpenInNew
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.Font
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.common.Modal
import net.nymtech.nymvpn.ui.common.buttons.MainStyledButton
import net.nymtech.nymvpn.ui.theme.CustomTypography
import net.nymtech.nymvpn.util.extensions.scaledHeight

@Composable
fun ServerDetailsModalBody(showLocationTooltip: Boolean, onClick: () -> Unit, onDismiss: () -> Unit) {
	Modal(
		show = showLocationTooltip,
		onDismiss = onDismiss,
		title = {
			Text(
				text = stringResource(R.string.gateway_locations_title),
				style = CustomTypography.labelHuge,
				fontFamily = FontFamily(Font(R.font.lab_grotesque_regular)),
			)
		},
		text = {
			Column {
				Text(
					stringResource(R.string.gateway_modal_description),
					textAlign = TextAlign.Center,
					style = MaterialTheme.typography.bodyMedium,
					fontFamily = FontFamily(Font(R.font.lab_grotesque_regular)),
				)
				Row(
					horizontalArrangement = Arrangement.spacedBy(2.dp, Alignment.CenterHorizontally),
					modifier = Modifier
						.fillMaxWidth()
						.padding(top = 12.dp)
						.clickable {
							onClick()
						},
				) {
					Text(
						stringResource(id = R.string.continue_reading),
						style = MaterialTheme.typography.bodyMedium,
						color = MaterialTheme.colorScheme.primary,
					)
					val icon = Icons.AutoMirrored.Outlined.OpenInNew
					Icon(
						icon,
						stringResource(R.string.go),
						Modifier
							.size(16.dp)
							.align(Alignment.CenterVertically),
						tint = MaterialTheme.colorScheme.primary,
					)
				}
			}
		},
		confirmButton = {
			MainStyledButton(
				onClick = onDismiss,
				textColor = Color.Black,
				content = { Text(stringResource(R.string.okay), fontFamily = FontFamily(Font(R.font.lab_grotesque_regular))) },
				modifier = Modifier
					.fillMaxWidth()
					.height(40.dp.scaledHeight()),
			)
		},
	)
}
