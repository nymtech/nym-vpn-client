package net.nymtech.nymvpn.ui.screens.details.components

import androidx.compose.foundation.Image
import androidx.compose.foundation.layout.size
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalInspectionMode
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.unit.Dp
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.theme.iconSize
import net.nymtech.nymvpn.util.extensions.getFlagImageVectorByName

@Composable
fun CountryFlag(countryCode: String?, size: Dp = iconSize) {
	val context = LocalContext.current
	val (painter, description) = if (LocalInspectionMode.current) {
		Pair(
			painterResource(R.drawable.flag_ua),
			"Unknown Country",
		)
	} else {
		countryCode?.let { code ->
			Pair(
				painterResource(id = context.getFlagImageVectorByName(code)),
				stringResource(R.string.country_flag, code),
			)
		} ?: Pair(
			painterResource(R.drawable.faq),
			stringResource(R.string.unknown),
		)
	}

	Image(
		painter,
		description,
		modifier = Modifier.size(size),
	)
}
