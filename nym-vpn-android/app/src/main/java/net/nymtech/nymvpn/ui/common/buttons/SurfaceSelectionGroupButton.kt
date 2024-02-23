package net.nymtech.nymvpn.ui.common.buttons

import androidx.compose.foundation.clickable
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.material3.Card
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.Divider
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.res.vectorResource
import androidx.compose.ui.unit.dp
import androidx.window.core.layout.WindowHeightSizeClass
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.MainActivity
import net.nymtech.nymvpn.ui.theme.iconSize
import net.nymtech.nymvpn.util.scaledHeight
import net.nymtech.nymvpn.util.scaledWidth

data class SelectionItem(
    val leadingIcon: ImageVector? = null,
    val trailing: (@Composable () -> Unit)? = {
      Icon(ImageVector.vectorResource(R.drawable.link_arrow_right), "arrow", Modifier.size(iconSize))
    },
    val title: String = "",
    val description: String? = null,
    val onClick: () -> Unit = {},
)

@Composable
fun SurfaceSelectionGroupButton(items: List<SelectionItem>) {
  val interactionSource = remember { MutableInteractionSource() }
  Card(
      modifier = Modifier.fillMaxWidth(),
      colors = CardDefaults.cardColors(containerColor = MaterialTheme.colorScheme.surface)) {
        items.mapIndexed { index, it ->
          Box(
              contentAlignment = Alignment.Center,
              modifier =
                  Modifier.clickable(
                      interactionSource = interactionSource, indication = null) {
                        it.onClick()
                      },
          ) {
            Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.Center, modifier = Modifier.height(64.dp.scaledHeight()).padding(top = 4.dp.scaledHeight(), bottom = 4.dp.scaledHeight(), end = 24.dp.scaledWidth())) {
              it.leadingIcon?.let { icon ->
                Icon(icon, icon.name, modifier = Modifier.padding(start = 16.dp.scaledWidth()).size(
                    iconSize))
              }
              Row(
                  horizontalArrangement = Arrangement.spacedBy(16.dp.scaledHeight()),
                  verticalAlignment = Alignment.CenterVertically) {
                    Column {
                      Text(it.title, style = MaterialTheme.typography.bodyLarge, modifier = Modifier.padding(start = 16.dp.scaledWidth()))
                      it.description?.let { description ->
                          val descriptionTypography = MaterialTheme.typography.bodyMedium
                        Text(
                            description,
                            color = MaterialTheme.colorScheme.onSurfaceVariant,
                            style = descriptionTypography,
                            modifier = Modifier.padding(start = 16.dp.scaledWidth()))
                      }
                    }
                  }
              Row(modifier = Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.End) {
                it.trailing?.let {
                    it()
                }
              }
            }
          }
          if (index + 1 != items.size) HorizontalDivider()
        }
      }
}
