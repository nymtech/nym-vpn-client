package net.nymtech.qrscanner

import android.graphics.Bitmap
import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.aspectRatio
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.FilterQuality
import androidx.compose.ui.graphics.ImageBitmap
import androidx.compose.ui.graphics.asImageBitmap
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.unit.dp
import com.google.zxing.BarcodeFormat
import com.google.zxing.EncodeHintType
import com.google.zxing.qrcode.QRCodeWriter
import com.google.zxing.qrcode.decoder.ErrorCorrectionLevel

// Always black on white: scanners need the contrast and a light quiet zone.
@Composable
fun QrCodeImage(content: String, contentDescription: String?, modifier: Modifier = Modifier) {
	val bitmap = remember(content) { QrCodeBitmap.encode(content) }
	Image(
		bitmap = bitmap,
		contentDescription = contentDescription,
		contentScale = ContentScale.Fit,
		filterQuality = FilterQuality.None,
		modifier = modifier
			.aspectRatio(1f)
			.background(Color.White, RoundedCornerShape(12.dp))
			.padding(12.dp),
	)
}

internal object QrCodeBitmap {
	private const val QUIET_ZONE_MODULES = 2
	private const val DARK = android.graphics.Color.BLACK
	private const val LIGHT = android.graphics.Color.WHITE

	fun encode(content: String): ImageBitmap {
		val hints = mapOf(
			EncodeHintType.ERROR_CORRECTION to ErrorCorrectionLevel.M,
			EncodeHintType.CHARACTER_SET to Charsets.UTF_8.name(),
			EncodeHintType.MARGIN to QUIET_ZONE_MODULES,
		)
		// Size 0 gives one pixel per module; FilterQuality.None scales it up sharply.
		val matrix = QRCodeWriter().encode(content, BarcodeFormat.QR_CODE, 0, 0, hints)
		val width = matrix.width
		val height = matrix.height
		val pixels = IntArray(width * height) { index -> if (matrix.get(index % width, index / width)) DARK else LIGHT }
		return Bitmap.createBitmap(pixels, width, height, Bitmap.Config.ARGB_8888).asImageBitmap()
	}
}
