package net.nymtech.nymvpn.ui.common.navigation

import androidx.compose.material3.Icon
import androidx.compose.runtime.Composable
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.res.vectorResource
import net.nymtech.nymvpn.R

@Composable
fun MainTitle() {
	return Icon(ImageVector.vectorResource(R.drawable.app_label), "app_label")
}
