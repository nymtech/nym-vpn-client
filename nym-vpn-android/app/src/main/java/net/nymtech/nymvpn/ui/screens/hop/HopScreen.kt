package net.nymtech.nymvpn.ui.screens.hop

import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.Search
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.res.vectorResource
import androidx.compose.ui.unit.dp
import androidx.hilt.navigation.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import androidx.navigation.NavController
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.AppViewModel
import net.nymtech.nymvpn.ui.HopType
import net.nymtech.nymvpn.ui.NavItem
import net.nymtech.nymvpn.ui.common.buttons.SelectionItemButton
import net.nymtech.nymvpn.ui.common.textbox.CustomTextField
import net.nymtech.nymvpn.ui.theme.iconSize
import net.nymtech.nymvpn.util.StringUtils
import net.nymtech.nymvpn.util.scaledHeight
import net.nymtech.nymvpn.util.scaledWidth
import net.nymtech.vpn.model.Country

@Composable
fun HopScreen(navController: NavController, hopType: HopType, appViewModel: AppViewModel, viewModel: HopViewModel = hiltViewModel()) {
	val uiState by viewModel.uiState.collectAsStateWithLifecycle()
	val context = LocalContext.current

	val countryComparator = compareBy<Country> { it.name }

	val sortedCountries =
		remember(uiState.queriedCountries, countryComparator) {
			uiState.queriedCountries.sortedWith(countryComparator)
		}

	LaunchedEffect(Unit) {
		viewModel.init(hopType)
		appViewModel.updateCountryListCache()
	}

	LazyColumn(
		horizontalAlignment = Alignment.CenterHorizontally,
		verticalArrangement = Arrangement.Top,
		modifier =
		Modifier
			.fillMaxSize()
			.padding(vertical = 24.dp.scaledHeight()),
	) {
		item {
			Column(
				verticalArrangement = Arrangement.spacedBy(24.dp.scaledHeight()),
				modifier = Modifier.padding(bottom = 24.dp.scaledHeight()).padding(horizontal = 24.dp.scaledWidth()),
			) {
				Box(
					modifier =
					Modifier
						.fillMaxWidth()
						.padding(
							horizontal = 16.dp.scaledWidth(),
							vertical = 16.dp.scaledHeight(),
						),
				)
				var query: String by rememberSaveable { mutableStateOf("") }
				CustomTextField(
					value = query,
					onValueChange = {
						query = it
						viewModel.onQueryChange(it)
					},
					modifier = Modifier
						.fillMaxWidth()
						.background(color = Color.Transparent, RoundedCornerShape(30.dp)),
					placeholder = {
						Text(
							stringResource(id = R.string.search_country),
							color = MaterialTheme.colorScheme.outline,
							style = MaterialTheme.typography.bodyLarge,
						)
					},
					singleLine = true,
					leading = {
						val icon = Icons.Rounded.Search
						Icon(
							imageVector = icon,
							modifier = Modifier.size(iconSize),
							tint = MaterialTheme.colorScheme.onBackground,
							contentDescription = icon.name,
						)
					},
					label = {
						Text(
							stringResource(R.string.search),
						)
					},
					textStyle = MaterialTheme.typography.bodyLarge.copy(
						color = MaterialTheme.colorScheme.onSurface,
					),
				)
			}
		}
		item {
			if (uiState.countries.isNotEmpty()) {
				val fastest = uiState.countries.firstOrNull { it.isLowLatency }
				if (fastest != null) {
					val name = StringUtils.buildCountryNameString(fastest, context)
					val icon = ImageVector.vectorResource(R.drawable.bolt)
					SelectionItemButton(
						{
							Icon(
								icon,
								icon.name,
								modifier =
								Modifier
									.padding(
										vertical = 16.dp.scaledHeight(),
										horizontal = 16.dp.scaledWidth(),
									)
									.size(
										iconSize,
									),
								tint = MaterialTheme.colorScheme.onSurface,
							)
						},
						name,
						onClick = {
							viewModel.onSelected(fastest)
							navController.navigate(NavItem.Main.route)
						},
						trailingText =
						if (fastest == uiState.selected) {
							stringResource(id = R.string.is_selected)
						} else {
							null
						},
					)
				}
			}
		}
		items(sortedCountries) {
			if (it.isLowLatency) return@items
			val icon =
				ImageVector.vectorResource(
					StringUtils.getFlagImageVectorByName(
						context,
						it.isoCode.lowercase(),
					),
				)
			SelectionItemButton(
				{
					Image(
						icon,
						icon.name,
						modifier =
						Modifier
							.padding(horizontal = 24.dp.scaledWidth(), 16.dp.scaledHeight())
							.size(
								iconSize,
							),
					)
				},
				buttonText = it.name,
				onClick = {
					viewModel.onSelected(it)
					navController.navigate(NavItem.Main.route)
				},
				trailingText =
				if (it == uiState.selected) stringResource(id = R.string.is_selected) else null,
			)
		}
	}
}
