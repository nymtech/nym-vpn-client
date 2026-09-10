package net.nymtech.nymvpn.ui.screens.settings.profiles

import android.content.res.Configuration
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.outlined.OpenInNew
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.style.TextDecoration
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.AppUiState
import net.nymtech.nymvpn.ui.AppViewModel
import net.nymtech.nymvpn.ui.screens.main.profiles.Profile
import net.nymtech.nymvpn.ui.screens.main.profiles.ProfileRow
import net.nymtech.nymvpn.ui.theme.NymVPNTheme
import net.nymtech.nymvpn.ui.theme.Theme
import net.nymtech.nymvpn.util.extensions.openWebUrl
import net.nymtech.nymvpn.util.extensions.scaledHeight
import net.nymtech.nymvpn.util.extensions.scaledWidth
import kotlin.enums.enumEntries

@Composable
fun ProfilesScreen(appUiState: AppUiState, appViewModel: AppViewModel) {
	ProfilesScreen(appUiState.currentProfile, appViewModel::onProfileSelected)
}

@Composable
fun ProfilesScreen(selected: Profile?, onSelect: (Profile) -> Unit) {
	val context = LocalContext.current
	val interactionSource = remember { MutableInteractionSource() }

	Column(
		horizontalAlignment = Alignment.Start,
		verticalArrangement = Arrangement.spacedBy(18.dp, Alignment.Top),
		modifier = Modifier
			.verticalScroll(rememberScrollState())
			.fillMaxSize()
			.padding(vertical = 18.dp.scaledHeight())
			.padding(horizontal = 18.dp.scaledWidth()),
	) {
		Text(
			text = stringResource(R.string.profiles_description),
			style = MaterialTheme.typography.bodyMedium.copy(MaterialTheme.colorScheme.onBackground),
		)
		Column(
			modifier = Modifier.fillMaxWidth()
				.background(MaterialTheme.colorScheme.surfaceVariant),
		) {
			enumEntries<Profile>().forEach { profile ->
				ProfileRow(
					profile = profile,
					selected = profile == selected,
					onClick = { onSelect(profile) },
					modifier = Modifier.fillMaxWidth(),
				)
			}
		}
		Row(
			horizontalArrangement = Arrangement.spacedBy(4.dp, Alignment.Start),
			modifier = Modifier
				.fillMaxWidth()
				.clickable(
					interactionSource = interactionSource,
					indication = null,
				) {
					context.openWebUrl(context.getString(R.string.profiles_learn_more_link))
				},
		) {
			Text(
				text = stringResource(R.string.profiles_learn_more_text),
				color = MaterialTheme.colorScheme.onPrimaryContainer,
				style = MaterialTheme.typography.bodyMedium.copy(
					textDecoration = TextDecoration.Underline,
				),
			)
			Icon(
				Icons.AutoMirrored.Outlined.OpenInNew,
				stringResource(R.string.go),
				Modifier
					.size(16.dp)
					.align(Alignment.CenterVertically),
				tint = MaterialTheme.colorScheme.onPrimaryContainer,
			)
		}
	}
}

@Composable
@Preview(uiMode = Configuration.UI_MODE_NIGHT_YES)
internal fun PreviewProfilesScreen() {
	NymVPNTheme(Theme.default()) {
		ProfilesScreen(null, {})
	}
}
