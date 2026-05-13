package net.nymtech.nymvpn.ui.screens.settings.tunneling.components

import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.outlined.CallSplit
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.Font
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.tooling.preview.PreviewLightDark
import androidx.compose.ui.unit.dp
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.common.Modal
import net.nymtech.nymvpn.ui.common.buttons.MainStyledButton
import net.nymtech.nymvpn.ui.common.buttons.TransparentButton
import net.nymtech.nymvpn.ui.theme.CustomTypography
import net.nymtech.nymvpn.ui.theme.NymVPNTheme
import net.nymtech.nymvpn.ui.theme.Theme
import net.nymtech.nymvpn.util.extensions.scaledHeight

@Composable
internal fun SplitTunnelingSettingModal(showModal: Boolean, onApplyNowClick: () -> Unit, onNextConnectionApplyClick: () -> Unit) {
	Modal(
		show = showModal,
		onDismiss = {},
		title = {
			Text(
				text = stringResource(R.string.split_tunnel_changes_confirm_modal_title),
				style = CustomTypography.titleMediumPlus,
				fontFamily = FontFamily(Font(R.font.lab_grotesque_regular)),
			)
		},
		text = {
			Column {
				Text(
					text = stringResource(R.string.split_tunnel_changes_confirm_modal_desc),
					style = MaterialTheme.typography.bodyMedium,
					textAlign = TextAlign.Center,
					fontFamily = FontFamily(Font(R.font.lab_grotesque_regular)),
				)
			}
		},
		confirmButton = {
			MainStyledButton(
				onClick = onApplyNowClick,
				textColor = Color.Black,
				content = {
					Text(
						stringResource(R.string.split_tunnel_changes_confirm_modal_primary_btn_label),
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
				onClick = onNextConnectionApplyClick,
				content = {
					Text(
						stringResource(R.string.split_tunnel_changes_confirm_modal_text_btn_label),
						fontFamily = FontFamily(Font(R.font.lab_grotesque_regular)),
						color = MaterialTheme.colorScheme.onSurface,
					)
				},
				modifier = Modifier
					.fillMaxWidth()
					.height(40.dp.scaledHeight()),
			)
		},
		icon = Icons.AutoMirrored.Outlined.CallSplit,
	)
}

@Composable
@PreviewLightDark
private fun ExitServerDetailsModalPreview() {
	NymVPNTheme(Theme.default()) {
		SplitTunnelingSettingModal(true, onApplyNowClick = {}, onNextConnectionApplyClick = {})
	}
}
