package net.nymtech.qrscanner.camera

import androidx.camera.core.ImageAnalysis
import androidx.camera.core.ImageProxy
import net.nymtech.qrscanner.decoder.LuminanceFrame
import net.nymtech.qrscanner.decoder.QrDecoder
import java.util.concurrent.atomic.AtomicBoolean
import kotlin.math.min

// Frames are copied to a reused in-memory buffer and closed immediately; only the scan area is decoded.
internal class QrCodeAnalyzer(private val decoder: QrDecoder, private val onCodeScan: (String) -> Unit) : ImageAnalysis.Analyzer {
	private val active = AtomicBoolean(true)
	private var luminance = ByteArray(0)

	override fun analyze(image: ImageProxy) {
		val frame = image.use { if (active.get()) it.toLuminanceFrame() else null } ?: return
		val text = decoder.decode(frame) ?: return
		if (active.get()) onCodeScan(text)
	}

	fun stop() {
		active.set(false)
	}

	fun wipe() {
		luminance.fill(0)
		luminance = ByteArray(0)
	}

	private fun ImageProxy.toLuminanceFrame(): LuminanceFrame {
		// Plane 0 of YUV_420_888 is luminance; rows may be padded.
		val plane = planes[0]
		val rowStride = plane.rowStride
		val required = rowStride * height
		if (luminance.size != required) {
			luminance.fill(0)
			luminance = ByteArray(required)
		}
		val buffer = plane.buffer.apply { rewind() }
		buffer.get(luminance, 0, min(buffer.remaining(), required))

		// With the viewport applied, cropRect is what the user sees.
		val crop = cropRect
		val side = (min(crop.width(), crop.height()) * ScanArea.ANALYSIS_FRACTION).toInt()
		return LuminanceFrame(
			data = luminance,
			dataWidth = rowStride,
			dataHeight = height,
			left = crop.centerX() - side / 2,
			top = crop.centerY() - side / 2,
			width = side,
			height = side,
		)
	}
}
