package net.nymtech.nymvpn.ui.screens.details.components

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Circle
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.graphics.vector.rememberVectorPainter
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.res.vectorResource
import androidx.compose.ui.text.SpanStyle
import androidx.compose.ui.text.buildAnnotatedString
import androidx.compose.ui.text.font.Font
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.style.TextDecoration
import androidx.compose.ui.text.withStyle
import androidx.compose.ui.tooling.preview.PreviewLightDark
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.theme.LocalNymColors
import net.nymtech.nymvpn.ui.theme.NymVPNTheme
import net.nymtech.nymvpn.ui.theme.Theme
import net.nymtech.nymvpn.ui.theme.Typography
import nym_vpn_lib_types.AsnKind

@Composable
fun DetailsSectionPrivacy(
	asnKind: AsnKind?,
	isQuicSupportedByGateway: Boolean,
	isPostQuantumEnabled: Boolean,
	nodeFamilyName: String?,
	isQuicEnabledLocally: Boolean,
	onEnableQuicProtocolClick: () -> Unit,
) {
	val items = buildList<Pair<String, @Composable () -> Unit>> {
		add(stringResource(R.string.details_advanced_privacy) to { MixnetItem() })

		add(stringResource(R.string.details_streaming_content) to { AsnKindItem(asnKind) })

		add(stringResource(R.string.details_post_quantum_secure_keys) to { PostQuantumItem(isPostQuantumEnabled) })

		nodeFamilyName?.let { name ->
			add(stringResource(R.string.details_family_membership) to { FamilyMembershipItem(name) })
		}

		add(stringResource(R.string.details_anti_censorship) to { QuicProtocolItem(isQuicSupportedByGateway) })
	}

	InfoSection(
		titleResId = R.string.details_features_title,
		items = items,
		bottomContent = {
			if (isQuicSupportedByGateway && !isQuicEnabledLocally) {
				QuicBottomContent(onEnableQuicProtocolClick)
			}
		},
	)
}

@Composable
private fun DetailsRow(icon: ImageVector, iconTint: Color, iconSize: Dp, textResId: Int) {
	DetailsRow(icon = icon, iconTint = iconTint, iconSize = iconSize, text = stringResource(textResId))
}

@Composable
private fun DetailsRow(icon: ImageVector, iconTint: Color, iconSize: Dp, text: String) {
	Row(verticalAlignment = Alignment.CenterVertically) {
		Icon(
			painter = rememberVectorPainter(icon),
			contentDescription = null,
			tint = iconTint,
			modifier = Modifier.size(iconSize),
		)
		Spacer(modifier = Modifier.width(6.dp))
		Text(
			text = text,
			style = Typography.bodyMedium,
			color = MaterialTheme.colorScheme.onPrimaryContainer,
			fontFamily = FontFamily(Font(R.font.lab_grotesque_regular)),
		)
	}
}

@Composable
private fun MixnetItem() {
	DetailsRow(
		icon = ImageVector.vectorResource(R.drawable.ic_mixnet),
		iconTint = MaterialTheme.colorScheme.primary,
		iconSize = 16.dp,
		textResId = R.string.details_with_mixnet,
	)
}

@Composable
private fun AsnKindItem(kind: AsnKind?) {
	val isResidential = kind == AsnKind.RESIDENTIAL
	DetailsRow(
		icon = if (isResidential) ImageVector.vectorResource(R.drawable.smart_display) else ImageVector.vectorResource(R.drawable.ic_database),
		iconTint = if (isResidential) Color.Unspecified else MaterialTheme.colorScheme.onBackground,
		iconSize = 20.dp,
		textResId = if (isResidential) R.string.details_residental_ip else R.string.details_datacenter_ip,
	)
}

@Composable
private fun PostQuantumItem(isEnabled: Boolean) {
	DetailsRow(
		icon = if (isEnabled) ImageVector.vectorResource(R.drawable.ic_quantum) else Icons.Filled.Circle,
		iconTint = if (isEnabled) Color.Unspecified else LocalNymColors.current.warning,
		iconSize = if (isEnabled) 20.dp else 12.dp,
		textResId = if (isEnabled) R.string.details_lewes_protocol else R.string.details_standard_key_exchange,
	)
}

@Composable
private fun FamilyMembershipItem(name: String) {
	DetailsRow(
		icon = ImageVector.vectorResource(R.drawable.ic_family),
		iconTint = MaterialTheme.colorScheme.primary,
		iconSize = 16.dp,
		text = name,
	)
}

@Composable
private fun QuicProtocolItem(isQuicSupported: Boolean) {
	DetailsRow(
		icon = if (isQuicSupported) ImageVector.vectorResource(R.drawable.quic_label) else Icons.Filled.Circle,
		iconTint = if (isQuicSupported) Color.Unspecified else LocalNymColors.current.warning,
		iconSize = if (isQuicSupported) 20.dp else 12.dp,
		textResId = if (isQuicSupported) R.string.details_quic_protocol else R.string.details_standard_protocol,
	)
}

@Composable
private fun QuicBottomContent(onEnableQuicProtocolClick: () -> Unit) {
	val annotatedText = buildAnnotatedString {
		pushStringAnnotation(tag = "QUIC", annotation = "quic_action")
		withStyle(
			style = SpanStyle(
				color = MaterialTheme.colorScheme.onPrimaryContainer,
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
			color = MaterialTheme.colorScheme.onBackground,
			fontFamily = FontFamily(Font(R.font.lab_grotesque_regular)),
		),
		modifier = Modifier.clickable {
			onEnableQuicProtocolClick()
		},
	)
}

@Composable
@PreviewLightDark
private fun PreviewDetailsSectionPrivacy() {
	NymVPNTheme(Theme.default()) {
		Surface {
			DetailsSectionPrivacy(
				asnKind = AsnKind.RESIDENTIAL,
				isQuicSupportedByGateway = true,
				isPostQuantumEnabled = true,
				nodeFamilyName = "Nym Family",
				isQuicEnabledLocally = false,
				onEnableQuicProtocolClick = {},
			)
		}
	}
}
