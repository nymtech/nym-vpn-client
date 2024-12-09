package net.nymtech.nymvpn.ui.common.navigation

import androidx.compose.foundation.isSystemInDarkTheme
import androidx.compose.material3.Icon
import androidx.compose.runtime.Composable
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.res.vectorResource
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.theme.Theme

@Composable
fun MainTitle(theme: Theme) {
	val darkTheme =
		when (theme) {
			Theme.AUTOMATIC -> isSystemInDarkTheme()
			Theme.DARK_MODE -> true
			Theme.LIGHT_MODE -> false
			else -> true
		}
	if (darkTheme) return Icon(ImageVector.vectorResource(R.drawable.app_label_dark), "app_label", tint = Color.Unspecified)
	Icon(ImageVector.vectorResource(R.drawable.app_label_light), "app_label", tint = Color.Unspecified)
}
