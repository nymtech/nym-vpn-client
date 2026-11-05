package net.nymtech.qrscanner.ui.components

import androidx.annotation.StringRes
import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.foundation.Canvas
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.BoxWithConstraints
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.navigationBarsPadding
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.geometry.CornerRadius
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.geometry.Rect
import androidx.compose.ui.geometry.Size
import androidx.compose.ui.graphics.BlendMode
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.CompositingStrategy
import androidx.compose.ui.graphics.Path
import androidx.compose.ui.graphics.StrokeCap
import androidx.compose.ui.graphics.drawscope.DrawScope
import androidx.compose.ui.graphics.drawscope.Stroke
import androidx.compose.ui.graphics.drawscope.rotate
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.semantics.LiveRegionMode
import androidx.compose.ui.semantics.liveRegion
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.min
import net.nymtech.qrscanner.camera.ScanArea
import net.nymtech.qrscanner.ui.theme.QrScannerColors

private val WindowCornerRadius = 24.dp
private val BracketLength = 40.dp
private val BracketStrokeWidth = 4.dp

@Composable
internal fun QrScannerOverlay(@StringRes hintRes: Int, @StringRes invalidCodeRes: Int, showInvalidCode: Boolean, @StringRes fallbackRes: Int, onFallback: () -> Unit, modifier: Modifier = Modifier) {
	val bracketColor = MaterialTheme.colorScheme.primary
	BoxWithConstraints(modifier = modifier.fillMaxSize()) {
		val windowSize = min(maxWidth, maxHeight) * ScanArea.WINDOW_FRACTION

		Canvas(
			modifier = Modifier
				.fillMaxSize()
				// Offscreen so the cleared window shows the preview through the scrim.
				.graphicsLayer { compositingStrategy = CompositingStrategy.Offscreen },
		) {
			val side = windowSize.toPx()
			val topLeft = Offset((size.width - side) / 2, (size.height - side) / 2)
			val cornerRadius = WindowCornerRadius.toPx()
			drawRect(QrScannerColors.scrim)
			drawRoundRect(
				color = Color.Transparent,
				topLeft = topLeft,
				size = Size(side, side),
				cornerRadius = CornerRadius(cornerRadius),
				blendMode = BlendMode.Clear,
			)
			drawCornerBrackets(
				topLeft = topLeft,
				side = side,
				color = bracketColor,
				radius = cornerRadius,
				length = BracketLength.toPx(),
				strokeWidth = BracketStrokeWidth.toPx(),
			)
		}

		// Equal weights keep the window centred, matching the canvas cut-out.
		Column(
			modifier = Modifier.fillMaxSize(),
			horizontalAlignment = Alignment.CenterHorizontally,
		) {
			Spacer(modifier = Modifier.weight(1f))
			Spacer(modifier = Modifier.size(windowSize))
			Column(
				modifier = Modifier
					.weight(1f)
					.fillMaxWidth()
					.navigationBarsPadding()
					.padding(horizontal = 24.dp, vertical = 24.dp),
				horizontalAlignment = Alignment.CenterHorizontally,
			) {
				Text(
					text = stringResource(hintRes),
					style = MaterialTheme.typography.bodyLarge,
					color = QrScannerColors.onOverlay,
					textAlign = TextAlign.Center,
				)
				AnimatedVisibility(
					visible = showInvalidCode,
					enter = fadeIn(),
					exit = fadeOut(),
				) {
					Text(
						text = stringResource(invalidCodeRes),
						style = MaterialTheme.typography.bodyMedium,
						color = MaterialTheme.colorScheme.onErrorContainer,
						textAlign = TextAlign.Center,
						modifier = Modifier
							.semantics { liveRegion = LiveRegionMode.Polite }
							.padding(top = 16.dp)
							.background(MaterialTheme.colorScheme.errorContainer, RoundedCornerShape(8.dp))
							.padding(horizontal = 12.dp, vertical = 6.dp),
					)
				}
				Spacer(modifier = Modifier.weight(1f))
				TextButton(onClick = onFallback) {
					Text(
						text = stringResource(fallbackRes),
						style = MaterialTheme.typography.labelLarge,
						color = QrScannerColors.onOverlay,
					)
				}
			}
		}
	}
}

private fun DrawScope.drawCornerBrackets(topLeft: Offset, side: Float, color: Color, radius: Float, length: Float, strokeWidth: Float) {
	val bracket = Path().apply {
		moveTo(topLeft.x, topLeft.y + length)
		lineTo(topLeft.x, topLeft.y + radius)
		arcTo(
			rect = Rect(topLeft, Size(radius * 2, radius * 2)),
			startAngleDegrees = 180f,
			sweepAngleDegrees = 90f,
			forceMoveTo = false,
		)
		lineTo(topLeft.x + length, topLeft.y)
	}
	val center = Offset(topLeft.x + side / 2, topLeft.y + side / 2)
	val stroke = Stroke(width = strokeWidth, cap = StrokeCap.Round)
	repeat(4) { quarter ->
		rotate(degrees = quarter * 90f, pivot = center) {
			drawPath(path = bracket, color = color, style = stroke)
		}
	}
}
