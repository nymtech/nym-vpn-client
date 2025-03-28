package net.nymtech.nymvpn.ui.screens.settings.support

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import net.nymtech.nymvpn.ui.screens.settings.support.components.SupportOptions
import net.nymtech.nymvpn.util.extensions.scaledHeight
import net.nymtech.nymvpn.util.extensions.scaledWidth

@Composable
fun SupportScreen() {
	Column(
		horizontalAlignment = Alignment.Start,
		verticalArrangement = Arrangement.spacedBy(24.dp, Alignment.Top),
		modifier = Modifier
			.verticalScroll(rememberScrollState())
			.fillMaxSize()
			.padding(top = 24.dp.scaledHeight())
			.padding(horizontal = 24.dp.scaledWidth()),
	) {
		SupportOptions()
	}
}
