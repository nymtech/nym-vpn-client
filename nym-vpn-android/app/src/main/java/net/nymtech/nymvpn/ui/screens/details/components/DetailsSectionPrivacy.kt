package net.nymtech.nymvpn.ui.screens.details.components

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Circle
import androidx.compose.material.icons.outlined.Check
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.vector.rememberVectorPainter
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.SpanStyle
import androidx.compose.ui.text.buildAnnotatedString
import androidx.compose.ui.text.font.Font
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.style.TextDecoration
import androidx.compose.ui.text.withStyle
import androidx.compose.ui.unit.dp
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.theme.CustomColors
import net.nymtech.nymvpn.ui.theme.Typography
import nym_vpn_lib_types.AsnKind

@Composable
fun DetailsSectionPrivacy(
	asnKind: AsnKind?,
	isQuicFeatureFlagEnabled: Boolean,
	isQuicSupportedByGateway: Boolean,
	onEnableQuicProtocolClicked: () -> Unit,
) {
	val items = buildList<Pair<String, @Composable () -> Unit>> {
		add(
			stringResource(R.string.details_advanced_privacy) to {
				Row(verticalAlignment = Alignment.CenterVertically) {
					Icon(
						painter = rememberVectorPainter(Icons.Outlined.Check),
						contentDescription = null,
						tint = Color.Green,
						modifier = Modifier.size(12.dp),
					)
					Spacer(modifier = Modifier.width(6.dp))
					Text(
						text = stringResource(R.string.details_with_mixnet),
						style = Typography.bodyMedium,
						color = MaterialTheme.colorScheme.onBackground,
						fontFamily = FontFamily(Font(R.font.lab_grotesque_regular)),
					)
				}
			},
		)

		asnKind?.let { kind ->
			add(
				stringResource(R.string.details_streaming_content) to {
					val isResidential = kind == AsnKind.RESIDENTIAL
					val icon = if (isResidential) Icons.Outlined.Check else Icons.Filled.Circle
					val iconTint = if (isResidential) Color.Green else CustomColors.warning
					val text = stringResource(
						if (isResidential) {
							R.string.details_residental_ip
						} else {
							R.string.details_datacenter_ip
						},
					)
					Row(verticalAlignment = Alignment.CenterVertically) {
						Icon(
							painter = rememberVectorPainter(icon),
							contentDescription = null,
							tint = iconTint,
							modifier = Modifier.size(12.dp),
						)
						Spacer(modifier = Modifier.width(6.dp))
						Text(
							text = text,
							style = Typography.bodyMedium,
							color = MaterialTheme.colorScheme.onBackground,
							fontFamily = FontFamily(Font(R.font.lab_grotesque_regular)),
						)
					}
				},
			)
		}

		if (isQuicFeatureFlagEnabled) {
			add(
				stringResource(R.string.details_anti_censorship) to {
					val icon = if (isQuicSupportedByGateway) Icons.Outlined.Check else Icons.Filled.Circle
					val iconTint = if (isQuicSupportedByGateway) Color.Green else CustomColors.warning
					val text = stringResource(
						if (isQuicSupportedByGateway) {
							R.string.details_quic_protocol
						} else {
							R.string.details_standard_protocol
						},
					)

					Row(verticalAlignment = Alignment.CenterVertically) {
						Icon(
							painter = rememberVectorPainter(icon),
							contentDescription = null,
							tint = iconTint,
							modifier = Modifier.size(12.dp),
						)
						Spacer(modifier = Modifier.width(6.dp))
						Text(
							text = text,
							style = Typography.bodyMedium,
							color = MaterialTheme.colorScheme.onBackground,
							fontFamily = FontFamily(Font(R.font.lab_grotesque_regular)),
						)
					}
				},
			)
		}
	}

	InfoSection(
		items = items,
		bottomContent = {
			if (isQuicFeatureFlagEnabled && isQuicSupportedByGateway) {
				val annotatedText = buildAnnotatedString {
					pushStringAnnotation(tag = "QUIC", annotation = "quic_action")
					withStyle(
						style = SpanStyle(
							color = MaterialTheme.colorScheme.onBackground,
							textDecoration = TextDecoration.Underline,
						),
					) {
						append(stringResource(R.string.details_enable_quic_start))
					}
					pop()
					append(" ")
					append(stringResource(R.string.details_enable_quic_end))
				}

				Text(
					text = annotatedText,
					style = Typography.labelSmall.copy(
						color = MaterialTheme.colorScheme.outline,
						fontFamily = FontFamily(Font(R.font.lab_grotesque_regular)),
					),
					modifier = Modifier.clickable {
						onEnableQuicProtocolClicked()
					},
				)
			}
		},
	)
}
