package net.nymtech.nymvpn.ui.screens.settings.appearance.language

import android.content.res.Configuration
import androidx.compose.foundation.clickable
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.IntrinsicSize
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.navigationBars
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.windowInsetsPadding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.outlined.Launch
import androidx.compose.material.icons.outlined.Language
import androidx.compose.material3.Card
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.AppUiState
import net.nymtech.nymvpn.ui.AppViewModel
import net.nymtech.nymvpn.ui.common.buttons.SelectionItemButton
import net.nymtech.nymvpn.ui.common.labels.SelectedLabel
import net.nymtech.nymvpn.ui.theme.NymVPNTheme
import net.nymtech.nymvpn.ui.theme.Theme
import net.nymtech.nymvpn.util.LocaleUtil
import net.nymtech.nymvpn.util.extensions.capitalize
import net.nymtech.nymvpn.util.extensions.openWebUrl
import net.nymtech.nymvpn.util.extensions.scaledHeight
import net.nymtech.nymvpn.util.extensions.scaledWidth
import java.util.Locale

@Composable
fun LanguageScreen(appUiState: AppUiState, appViewModel: AppViewModel) {
	val context = LocalContext.current
	val url = stringResource(R.string.settings_language_crowdin_link)

	LanguageScreen(
		locales = LocaleUtil.supportedLocales,
		currentLocale = appUiState.settings.locale,
		onLocaleChange = {
			appViewModel.onLocaleChange(it)
		},
		onHelpClick = {
			context.openWebUrl(url)
		},
	)
}

@Composable
fun LanguageScreen(locales: List<Locale>, currentLocale: String?, onLocaleChange: (locale: String) -> Unit, onHelpClick: () -> Unit) {
	val effectiveLocale = currentLocale?.takeIf { it.isNotBlank() } ?: LocaleUtil.OPTION_PHONE_LANGUAGE

	Column(
		modifier = Modifier
			.fillMaxSize()
			.padding(24.dp)
			.windowInsetsPadding(WindowInsets.navigationBars),
	) {
		val interactionSource = remember { MutableInteractionSource() }
		Card(
			modifier = Modifier.fillMaxWidth(),
			shape = RoundedCornerShape(8.dp),
			colors = CardDefaults.cardColors(containerColor = MaterialTheme.colorScheme.primaryContainer),
		) {
			Box(
				contentAlignment = Alignment.Center,
				modifier =
				Modifier
					.clickable(
						interactionSource = interactionSource,
						indication = null,
					) {
						onHelpClick.invoke()
					}
					.fillMaxWidth().height(IntrinsicSize.Min),
			) {
				Row(
					verticalAlignment = Alignment.CenterVertically,
					modifier = Modifier.fillMaxSize(),
				) {
					Row(
						verticalAlignment = Alignment.CenterVertically,
						modifier = Modifier
							.weight(1f, false)
							.padding(vertical = 8.dp.scaledHeight())
							.fillMaxSize()
							.padding(end = 4.dp.scaledWidth()),
					) {
						Box(modifier = Modifier.padding(start = 16.dp.scaledWidth()))
						Box(modifier = Modifier.padding(end = 16.dp.scaledWidth())) {
							Icon(
								Icons.Outlined.Language,
								stringResource(R.string.account),
								modifier = Modifier.size(24.dp.scaledWidth()),
								tint = MaterialTheme.colorScheme.onBackground,
							)
						}
						Column(
							horizontalAlignment = Alignment.Start,
							verticalArrangement = Arrangement.spacedBy(2.dp, Alignment.CenterVertically),
							modifier = Modifier
								.fillMaxWidth()
								.padding(vertical = 16.dp.scaledHeight()),
						) {
							Text(
								stringResource(R.string.settings_language_help_title),
								style = MaterialTheme.typography.bodyLarge,
								color = MaterialTheme.colorScheme.onPrimaryContainer,
							)
							Text(
								stringResource(R.string.settings_language_help_description),
								style = MaterialTheme.typography.bodySmall,
								color = MaterialTheme.colorScheme.onBackground,
							)
						}
					}

					Box(
						contentAlignment = Alignment.CenterEnd,
						modifier = Modifier
							.weight(0.35f)
							.padding(horizontal = 16.dp.scaledWidth()),
					) {
						Icon(
							Icons.AutoMirrored.Outlined.Launch,
							stringResource(R.string.go),
							modifier = Modifier.size(24.dp),
							tint = MaterialTheme.colorScheme.onBackground,
						)
					}
				}
			}
		}

		LazyColumn(
			horizontalAlignment = Alignment.CenterHorizontally,
			verticalArrangement = Arrangement.Top,
			modifier = Modifier
				.fillMaxWidth()
				.weight(1f),
		) {
			item {
				SelectionItemButton(
					buttonText = stringResource(R.string.automatic),
					onClick = {
						onLocaleChange(LocaleUtil.OPTION_PHONE_LANGUAGE)
					},
					trailing = {
						if (effectiveLocale == LocaleUtil.OPTION_PHONE_LANGUAGE) {
							SelectedLabel()
						}
					},
					ripple = false,
				)
			}
			items(locales, key = { it.toLanguageTag() }) { locale ->
				SelectionItemButton(
					buttonText = locale.getDisplayLanguage(locale).capitalize(locale) +
						if (locale.toLanguageTag().contains("-")) " (${locale.getDisplayCountry(locale).capitalize(locale)})" else "",
					onClick = {
						onLocaleChange(locale.toLanguageTag())
					},
					trailing = {
						if (locale.toLanguageTag() == effectiveLocale) {
							SelectedLabel()
						}
					},
					ripple = false,
				)
			}
		}
	}
}

@Composable
@Preview(uiMode = Configuration.UI_MODE_NIGHT_YES)
internal fun PreviewLanguageScreen() {
	NymVPNTheme(Theme.default()) {
		LanguageScreen(
			locales = listOf(
				Locale.CANADA,
				Locale.UK,
				Locale.CHINA,
				Locale.FRANCE,
				Locale.GERMANY,
			),
			currentLocale = "UK",
			onLocaleChange = {},
			onHelpClick = {},
		)
	}
}
