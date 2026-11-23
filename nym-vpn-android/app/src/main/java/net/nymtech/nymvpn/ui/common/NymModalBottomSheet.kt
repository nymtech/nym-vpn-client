package net.nymtech.nymvpn.ui.common

import android.os.Build
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.ColumnScope
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.BottomSheetDefaults
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.ModalBottomSheet
import androidx.compose.material3.ModalBottomSheetProperties
import androidx.compose.material3.SheetState
import androidx.compose.material3.rememberModalBottomSheetState
import androidx.compose.runtime.Composable
import androidx.compose.runtime.SideEffect
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.Shape
import androidx.compose.ui.platform.LocalView
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import androidx.compose.ui.window.DialogWindowProvider

/** ModalBottomSheet wrapper that removes the grey nav bar scrim on 3-button navigation. */

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun NymModalBottomSheet(
	onDismissRequest: () -> Unit,
	modifier: Modifier = Modifier,
	sheetState: SheetState = rememberModalBottomSheetState(),
	shape: Shape = BottomSheetDefaults.ExpandedShape,
	containerColor: Color = BottomSheetDefaults.ContainerColor,
	tonalElevation: Dp = 0.dp,
	dragHandle: @Composable (() -> Unit)? = { BottomSheetDefaults.DragHandle() },
	contentWindowInsets: @Composable () -> WindowInsets = { BottomSheetDefaults.windowInsets },
	properties: ModalBottomSheetProperties = ModalBottomSheetProperties(),
	content: @Composable ColumnScope.() -> Unit,
) {
	ModalBottomSheet(
		onDismissRequest = onDismissRequest,
		modifier = modifier,
		sheetState = sheetState,
		shape = shape,
		containerColor = containerColor,
		tonalElevation = tonalElevation,
		dragHandle = dragHandle,
		contentWindowInsets = contentWindowInsets,
		properties = properties,
	) {
		val view = LocalView.current
		if (!view.isInEditMode && Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q) {
			SideEffect {
				(view.parent as? DialogWindowProvider)?.window?.isNavigationBarContrastEnforced = false
			}
		}
		content()
	}
}

@Composable
fun NymSheetDragHandle(containerColor: Color = MaterialTheme.colorScheme.surface) {
	Box(
		modifier = Modifier
			.fillMaxWidth()
			.background(containerColor)
			.padding(top = 8.dp),
		contentAlignment = Alignment.Center,
	) {
		Box(
			modifier = Modifier
				.size(width = 32.dp, height = 4.dp)
				.clip(RoundedCornerShape(2.dp))
				.background(MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.4f)),
		)
	}
}
