package net.nymtech.nymvpn.ui.common.buttons

import androidx.compose.foundation.clickable
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Card
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import net.nymtech.nymvpn.util.scaledHeight
import net.nymtech.nymvpn.util.scaledWidth

@Composable
fun SelectionItemButton(
    leading: @Composable () -> Unit,
    buttonText: String,
    selected: Boolean,
    trailingText: String?,
    onClick: () -> Unit
) {
  Card(
      modifier =
          Modifier.clickable(
                  indication = null,
                  interactionSource = remember { MutableInteractionSource() },
                  onClick = { onClick() })
              .height(56.dp.scaledHeight()),
      shape = RoundedCornerShape(10.dp),
      colors = CardDefaults.cardColors(containerColor = if(!selected) MaterialTheme.colorScheme.background else MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha=0.16f))) {
        Row(
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.Start,
            modifier = Modifier.fillMaxWidth()) {
              leading()
              Text(
                  buttonText,
                  style = MaterialTheme.typography.bodyMedium,
                  color = MaterialTheme.colorScheme.onSurface)
            trailingText?.let {
                Row(modifier = Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.End,
                    verticalAlignment = Alignment.CenterVertically){
                    Text(it, modifier = Modifier.padding(horizontal = 16.dp.scaledWidth(), vertical = 16.dp.scaledHeight()), color =
                    MaterialTheme.colorScheme.onSurfaceVariant, style = MaterialTheme.typography.labelSmall)
                }
            }
            }

      }
}
