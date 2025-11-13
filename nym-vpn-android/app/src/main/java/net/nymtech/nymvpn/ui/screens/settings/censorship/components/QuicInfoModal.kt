package net.nymtech.nymvpn.ui.screens.settings.censorship.components

import androidx.annotation.StringRes
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.res.vectorResource
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
import net.nymtech.nymvpn.ui.theme.iconSize
import net.nymtech.nymvpn.util.extensions.scaledHeight

@Composable
fun QuicInfoModal(show: Boolean, data: QuicInfoModalData, onConfirm: () -> Unit, onDismiss: () -> Unit) {
	Modal(
		show = show,
		onDismiss = onDismiss,
		icon = ImageVector.vectorResource(R.drawable.quic_label),
		iconSize = iconSize.scaledHeight(),
		description = stringResource(R.string.quic),
		title = {
			Text(
				text = stringResource(data.title),
				style = CustomTypography.labelHuge,
				fontFamily = FontFamily(Font(R.font.lab_grotesque_regular)),
			)
		},
		text = {
			Text(
				text = stringResource(data.description),
				textAlign = TextAlign.Center,
				style = MaterialTheme.typography.bodyMedium,
				fontFamily = FontFamily(Font(R.font.lab_grotesque_regular)),
			)
		},
		confirmButton = {
			MainStyledButton(
				onClick = onConfirm,
				content = {
					Text(
						text = stringResource(data.primaryButtonText),
						fontFamily = FontFamily(Font(R.font.lab_grotesque_regular)),
						color = Color.Black,
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
						text = stringResource(data.secondaryButtonText),
						fontFamily = FontFamily(Font(R.font.lab_grotesque_regular)),
						color = MaterialTheme.colorScheme.onSurface,
					)
				},
				modifier = Modifier
					.fillMaxWidth()
					.height(40.dp.scaledHeight()),
			)
		},
	)
}

data class QuicInfoModalData(
	@param:StringRes val title: Int,
	@param:StringRes val description: Int,
	@param:StringRes val primaryButtonText: Int,
	@param:StringRes val secondaryButtonText: Int,
) {
	companion object {
		fun getQuicInfoModalData(connectionStatus: ConnectionStatus): QuicInfoModalData {
			return when (connectionStatus) {
				ConnectionStatus.FastNonQuicConnected -> {
					QuicInfoModalData(
						title = R.string.quick_info_modal_title,
						description = R.string.quick_info_modal_desc,
						primaryButtonText = R.string.quic_primary_btn_text,
						secondaryButtonText = R.string.quic_text_btn_text,
					)
				}

				else -> {
					QuicInfoModalData(
						title = R.string.quick_info_modal_title_non_quic,
						description = R.string.quick_info_modal_desc_non_quic,
						primaryButtonText = R.string.quic_primary_btn_text_non_quic,
						secondaryButtonText = R.string.quic_text_btn_text_non_quic,
					)
				}
			}
		}
	}
}

@Composable
@PreviewLightDark
internal fun QuicInfoModalPreview() {
	NymVPNTheme(Theme.default()) {
		QuicInfoModal(true, data = QuicInfoModalData.getQuicInfoModalData(ConnectionStatus.FastQuicConnected), {}, {})
	}
}
