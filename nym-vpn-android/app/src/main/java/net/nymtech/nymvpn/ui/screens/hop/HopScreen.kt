package net.nymtech.nymvpn.ui.screens.hop

import androidx.compose.foundation.Image
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.res.vectorResource
import androidx.compose.ui.unit.dp
import androidx.hilt.navigation.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import androidx.navigation.NavController
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.HopType
import net.nymtech.nymvpn.ui.NavItem
import net.nymtech.nymvpn.ui.common.SearchBar
import net.nymtech.nymvpn.ui.common.buttons.SelectionItemButton
import net.nymtech.nymvpn.ui.theme.screenPadding
import net.nymtech.nymvpn.util.StringUtils

@Composable
fun HopScreen(
    viewModel: HopViewModel = hiltViewModel(),
    navController: NavController,
    hopType: HopType
) {

  val uiState by viewModel.uiState.collectAsStateWithLifecycle()
  val context = LocalContext.current

  LaunchedEffect(Unit) {
      viewModel.init(hopType)
  }

  LazyColumn(
     horizontalAlignment = Alignment.CenterHorizontally,
      verticalArrangement = Arrangement.Top,
      modifier = Modifier.fillMaxSize().padding(screenPadding)) {
        item {
          Column(
              verticalArrangement = Arrangement.spacedBy(24.dp),
              modifier = Modifier.padding(bottom = 24.dp)) {
                if (uiState.countries.isNotEmpty()) {
                  val fastest = uiState.countries.firstOrNull { it.isFastest }
                  if (fastest != null) {
                    val name = StringUtils.buildCountryNameString(fastest, context)
                    val icon = ImageVector.vectorResource(R.drawable.bolt)
                    SelectionItemButton(
                        {
                          Icon(
                              icon,
                              icon.name,
                              modifier = Modifier.padding(16.dp),
                              tint = MaterialTheme.colorScheme.onSurface)
                        },
                        name,
                        onClick = {
                          viewModel.onSelected(fastest)
                          navController.navigate(NavItem.Main.route)
                        },
                        selected = fastest == uiState.selected,
                        trailingText =
                            if (fastest == uiState.selected)
                                stringResource(id = R.string.is_selected)
                            else null)
                  }
                }
                SearchBar(
                    onQuery = viewModel::onQueryChange,
                    placeholder = stringResource(id = R.string.search_country),
                    16.dp)
              }
        }
        items(uiState.queriedCountries) {
          if (it.isFastest) return@items
          val icon =
              ImageVector.vectorResource(StringUtils.getFlagImageVectorByName(context, it.isoCode.lowercase()))
          SelectionItemButton(
              { Image(icon, icon.name, modifier = Modifier.padding(16.dp)) },
              buttonText = it.name,
              onClick = {
                viewModel.onSelected(it)
                navController.navigate(NavItem.Main.route)
              },
              selected = it == uiState.selected,
              trailingText =
                  if (it == uiState.selected) stringResource(id = R.string.is_selected) else null)
        }
      }
}
