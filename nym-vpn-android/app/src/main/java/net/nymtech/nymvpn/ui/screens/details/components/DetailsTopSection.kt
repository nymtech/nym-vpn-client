package net.nymtech.nymvpn.ui.screens.details.components

import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.Font
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.unit.dp
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.theme.CustomTypography
import net.nymtech.nymvpn.ui.theme.Typography
import nym_vpn_lib_types.AsnKind

@Composable
internal fun DetailsTopSection(
	name: String,
	location: String,
	isQuicFeatureFlagEnabled: Boolean,
	isQuickSupportedByGateway: Boolean,
	countryCode: String?,
	description: String?,
	asnKind: AsnKind?,
	onEnableQuicProtocolClicked: () -> Unit,
) {
	Text(
		text = name,
		style = CustomTypography.titleMediumPlus,
		color = MaterialTheme.colorScheme.onBackground,
		fontFamily = FontFamily(Font(R.font.lab_grotesque_regular)),
	)
	Row(
		modifier = Modifier
			.padding(top = 16.dp),
		verticalAlignment = Alignment.CenterVertically,
	) {
		CountryFlag(countryCode, 16.dp)
		Spacer(modifier = Modifier.width(8.dp))
		Text(
			text = location,
			style = Typography.titleMedium,
			color = MaterialTheme.colorScheme.onBackground,
			fontFamily = FontFamily(Font(R.font.lab_grotesque_regular)),
		)
	}
	description?.let {
		Text(
			modifier = Modifier.padding(top = 16.dp),
			text = it,
			style = Typography.bodyMedium,
			color = MaterialTheme.colorScheme.outline,
			fontFamily = FontFamily(Font(R.font.lab_grotesque_regular)),
		)
	}
	DetailsSectionPrivacy(
		asnKind,
		isQuicFeatureFlagEnabled,
		isQuickSupportedByGateway,
		onEnableQuicProtocolClicked,
	)
}
