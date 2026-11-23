package net.nymtech.qrscanner.decoder

import com.google.zxing.BinaryBitmap
import com.google.zxing.DecodeHintType
import com.google.zxing.LuminanceSource
import com.google.zxing.PlanarYUVLuminanceSource
import com.google.zxing.ReaderException
import com.google.zxing.common.HybridBinarizer
import com.google.zxing.qrcode.QRCodeReader

// Not thread-safe.
internal class ZxingQrDecoder : QrDecoder {
	private val reader = QRCodeReader()
	private val hints = mapOf(DecodeHintType.CHARACTER_SET to Charsets.UTF_8.name())

	override fun decode(frame: LuminanceFrame): String? {
		val source = PlanarYUVLuminanceSource(
			frame.data,
			frame.dataWidth,
			frame.dataHeight,
			frame.left,
			frame.top,
			frame.width,
			frame.height,
			false,
		)
		// Dark themes may render the code light-on-dark.
		return decode(source) ?: decode(source.invert())
	}

	private fun decode(source: LuminanceSource): String? = try {
		reader.decode(BinaryBitmap(HybridBinarizer(source)), hints).text
	} catch (_: ReaderException) {
		null
	} finally {
		reader.reset()
	}
}
