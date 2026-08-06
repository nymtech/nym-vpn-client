package net.nymtech.nymvpn.ui.screens.details.components

import androidx.compose.foundation.Image
import androidx.compose.foundation.layout.size
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.ColorFilter
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalInspectionMode
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.unit.Dp
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.screens.main.panel.NodeSelectionType
import net.nymtech.nymvpn.ui.theme.iconSize
import net.nymtech.nymvpn.util.extensions.getFlagImageVectorByName

@Composable
fun CountryFlag(countryCode: String?, size: Dp = iconSize, selectionType: NodeSelectionType = NodeSelectionType.NODE) {
	val context = LocalContext.current
	val (painter, description, colorFilter) = if (LocalInspectionMode.current) {
		Triple(
			painterResource(R.drawable.flag_ua),
			"Unknown Country",
			null,
		)
	} else {
		countryCode?.let { code ->
			Triple(
				painterResource(id = context.getFlagImageVectorByName(code)),
				stringResource(R.string.country_flag, code),
				null,
			)
		} ?: when (selectionType) {
			NodeSelectionType.AUTO -> Triple(
				painterResource(R.drawable.ic_safest),
				stringResource(R.string.gateway_safest),
				null,
			)
			NodeSelectionType.RANDOM -> Triple(
				painterResource(R.drawable.ic_random),
				stringResource(R.string.unknown),
				ColorFilter.tint(MaterialTheme.colorScheme.onBackground),
			)
			NodeSelectionType.NODE -> Triple(
				painterResource(R.drawable.faq),
				stringResource(R.string.unknown),
				ColorFilter.tint(MaterialTheme.colorScheme.onBackground),
			)
		}
	}

	Image(
		painter,
		description,
		modifier = Modifier.size(size),
		colorFilter = colorFilter,
	)
}
