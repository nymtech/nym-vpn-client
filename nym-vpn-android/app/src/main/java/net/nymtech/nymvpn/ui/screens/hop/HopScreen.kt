package net.nymtech.nymvpn.ui.screens.hop

import androidx.compose.foundation.Image
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
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
import net.nymtech.nymvpn.ui.AppViewModel
import net.nymtech.nymvpn.ui.HopType
import net.nymtech.nymvpn.ui.NavItem
import net.nymtech.nymvpn.ui.common.SearchBar
import net.nymtech.nymvpn.ui.common.buttons.SelectionItemButton
import net.nymtech.nymvpn.ui.theme.iconSize
import net.nymtech.nymvpn.util.StringUtils
import net.nymtech.nymvpn.util.scaledHeight
import net.nymtech.nymvpn.util.scaledWidth

@Composable
fun HopScreen(
    navController: NavController,
    hopType: HopType,
    appViewModel: AppViewModel,
    viewModel: HopViewModel = hiltViewModel(),
) {

  val uiState by viewModel.uiState.collectAsStateWithLifecycle()
  val context = LocalContext.current

  LaunchedEffect(Unit) {
      viewModel.init(hopType)
      appViewModel.updateCountryListCache()
  }

  LazyColumn(
     horizontalAlignment = Alignment.CenterHorizontally,
      verticalArrangement = Arrangement.Top,
      modifier = Modifier
          .fillMaxSize()
          .padding(horizontal = 24.dp.scaledWidth(), vertical = 24.dp.scaledHeight())) {
        item {
          Column(
              verticalArrangement = Arrangement.spacedBy(24.dp.scaledHeight()),
              modifier = Modifier.padding(bottom = 24.dp.scaledHeight())) {
              Box(modifier = Modifier.fillMaxWidth().padding(horizontal = 16.dp.scaledWidth(), vertical = 16.dp.scaledHeight()))
                SearchBar(
                    onQuery = viewModel::onQueryChange,
                    placeholder = { Text(stringResource(id = R.string.search_country), color = MaterialTheme.colorScheme.outline) })
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
                                modifier = Modifier.padding(horizontal = 16.dp.scaledWidth(), 16.dp.scaledHeight()).size(
                                    iconSize),
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
        }
        items(uiState.queriedCountries.toList()) {
          if (it.isLowLatency) return@items
          val icon =
              ImageVector.vectorResource(StringUtils.getFlagImageVectorByName(context, it.isoCode.lowercase()))
          SelectionItemButton(
              { Image(icon, icon.name, modifier = Modifier.padding(horizontal = 16.dp.scaledWidth(), 16.dp.scaledHeight()).size(
                  iconSize)) },
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
