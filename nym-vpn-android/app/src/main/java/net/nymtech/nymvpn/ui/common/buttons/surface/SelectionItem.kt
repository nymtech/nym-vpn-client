package net.nymtech.nymvpn.ui.common.buttons.surface

import androidx.compose.foundation.layout.size
import androidx.compose.material3.Icon
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.res.vectorResource
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.theme.iconSize

data class SelectionItem(
    val leadingIcon: ImageVector? = null,
    val trailing: (@Composable () -> Unit)? = {
        Icon(
            ImageVector.vectorResource(R.drawable.link_arrow_right), "arrow", Modifier.size(
                iconSize
            )
        )
    },
    val title: String = "",
    val description: String? = null,
    val onClick: () -> Unit = {},
)