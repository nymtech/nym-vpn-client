package net.nymtech.nymvpn.ui.screens.hop

import androidx.compose.foundation.Image
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.outlined.ContentCopy
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.material3.VerticalDivider
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.platform.LocalClipboardManager
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.res.vectorResource
import androidx.compose.ui.text.AnnotatedString
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.common.buttons.MainStyledButton
import net.nymtech.nymvpn.ui.theme.CustomTypography
import net.nymtech.nymvpn.util.extensions.getFlagImageVectorByName
import net.nymtech.nymvpn.util.extensions.getScoreIcon
import net.nymtech.nymvpn.util.extensions.toLocale
import net.nymtech.vpn.model.NymGateway
import nym_vpn_lib.GatewayType

@Composable
fun GatewayDetailsModal(gateway: NymGateway, gatewayType: GatewayType, onDismiss: () -> Unit) {
	val context = LocalContext.current
	val clipboard = LocalClipboardManager.current

	AlertDialog(
		containerColor = MaterialTheme.colorScheme.surfaceContainer,
		onDismissRequest = { onDismiss() },
		tonalElevation = 0.dp,
		modifier = Modifier.width(312.dp),
		confirmButton = {
			MainStyledButton(
				onClick = {
					onDismiss()
				},
				content = {
					Text(text = stringResource(id = R.string.close))
				},
			)
		},
		title = {
			Column(
				horizontalAlignment = Alignment.Start,
				verticalArrangement = Arrangement.spacedBy(12.dp, Alignment.CenterVertically),
			) {
				Text(gateway.name, style = CustomTypography.labelHuge, textAlign = TextAlign.Start)
				Row(
					horizontalArrangement = Arrangement.spacedBy(8.dp),
					verticalAlignment = Alignment.CenterVertically,
				) {
					val icon = gateway.getScoreIcon(gatewayType)
					Image(
						icon,
						icon.name,
						modifier = Modifier.height(16.dp).width(15.dp),
					)
					VerticalDivider(modifier = Modifier.width(1.dp).size(24.dp))
					val countryIconResource = gateway.twoLetterCountryISO?.let {
						context.getFlagImageVectorByName(it)
					} ?: R.drawable.faq
					val countryIcon = ImageVector.vectorResource(countryIconResource)
					Image(
						countryIcon,
						countryIcon.name,
						modifier =
						Modifier
							.size(16.dp),
					)
					Text(gateway.toLocale()?.displayCountry ?: stringResource(R.string.unknown), style = MaterialTheme.typography.bodyLarge)
				}
			}
		},
		text = {
			Column(
				horizontalAlignment = Alignment.Start,
				verticalArrangement = Arrangement.spacedBy(8.dp, Alignment.CenterVertically),
				modifier = Modifier.padding(vertical = 12.dp),
			) {
				Text(stringResource(R.string.identity_key), style = MaterialTheme.typography.bodySmall, color = MaterialTheme.colorScheme.outline)
				Row(horizontalArrangement = Arrangement.spacedBy(16.dp)) {
					Text(
						gateway.identity,
						style = MaterialTheme.typography.bodyMedium.copy(color = MaterialTheme.colorScheme.onSurface),
						modifier = Modifier.width(232.dp),
					)
					val icon = Icons.Outlined.ContentCopy
					Icon(
						icon,
						contentDescription = icon.name,
						Modifier.size(16.dp).clickable {
							clipboard.setText(AnnotatedString(gateway.identity))
						},
						tint = MaterialTheme.colorScheme.primary,
					)
				}
			}
		},
	)
}
