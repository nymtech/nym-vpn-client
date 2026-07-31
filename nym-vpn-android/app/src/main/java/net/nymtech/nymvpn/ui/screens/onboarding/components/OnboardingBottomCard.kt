package net.nymtech.nymvpn.ui.screens.onboarding.components

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.res.vectorResource
import androidx.compose.ui.unit.dp
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.common.buttons.MainStyledButton
import net.nymtech.nymvpn.ui.theme.CustomTypography
import net.nymtech.nymvpn.util.extensions.scaledHeight

@Composable
fun OnboardingBottomCard(onGetStartedClick: () -> Unit, modifier: Modifier = Modifier, tagline: String? = null) {
	Column(
		horizontalAlignment = Alignment.CenterHorizontally,
		verticalArrangement = Arrangement.spacedBy(12.dp),
		modifier = modifier
			.fillMaxWidth()
			.background(
				color = MaterialTheme.colorScheme.surfaceContainer,
				shape = RoundedCornerShape(16.dp),
			)
			.padding(horizontal = 20.dp, vertical = 16.dp),
	) {
		Icon(
			imageVector = ImageVector.vectorResource(R.drawable.app_label),
			contentDescription = stringResource(R.string.app_name),
			tint = MaterialTheme.colorScheme.onSurface,
		)
		Text(
			text = tagline ?: " ",
			maxLines = 1,
			style = MaterialTheme.typography.bodyMedium,
			color = MaterialTheme.colorScheme.onBackground,
		)
		MainStyledButton(
			onClick = onGetStartedClick,
			content = {
				Text(
					text = stringResource(R.string.get_started),
					style = CustomTypography.buttonMain,
				)
			},
			color = MaterialTheme.colorScheme.primary,
			modifier = Modifier
				.fillMaxWidth()
				.height(48.dp.scaledHeight()),
			shape = RoundedCornerShape(12.dp),
		)
	}
}
