package net.nymtech.qrscanner.decoder

import com.google.zxing.BarcodeFormat
import com.google.zxing.qrcode.QRCodeWriter
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Test

class ZxingQrDecoderTest {
	private val decoder = ZxingQrDecoder()
	private val text = "https://nym.com/qr-pairing?pubkey=9Ez7Qw3hLk2sVtYpRbN4cJmA6uXfD8gH1iKoP5rS"

	@Test
	fun decodesCodeInsideSearchArea() {
		val frame = frameWithCode(text, codeLeft = 100, codeTop = 80)
		assertEquals(text, decoder.decode(frame.searching(left = 50, top = 30, size = 400)))
	}

	@Test
	fun decodesInvertedCode() {
		val frame = frameWithCode(text, codeLeft = 100, codeTop = 80, inverted = true)
		assertEquals(text, decoder.decode(frame.searching(left = 50, top = 30, size = 400)))
	}

	@Test
	fun decodesWithPaddedRows() {
		val frame = frameWithCode(text, codeLeft = 100, codeTop = 80, rowPadding = 64)
		assertEquals(text, decoder.decode(frame.searching(left = 50, top = 30, size = 400)))
	}

	@Test
	fun ignoresCodeOutsideSearchArea() {
		val frame = frameWithCode(text, codeLeft = 380, codeTop = 230)
		assertNull(decoder.decode(frame.searching(left = 0, top = 0, size = 300)))
	}

	@Test
	fun returnsNullForBlankFrame() {
		val frame = Frame(ByteArray(FRAME_WIDTH * FRAME_HEIGHT) { WHITE }, FRAME_WIDTH, FRAME_HEIGHT)
		assertNull(decoder.decode(frame.searching(left = 0, top = 0, size = FRAME_HEIGHT)))
	}

	private class Frame(val data: ByteArray, val dataWidth: Int, val dataHeight: Int) {
		fun searching(left: Int, top: Int, size: Int) = LuminanceFrame(data, dataWidth, dataHeight, left, top, size, size)
	}

	private fun frameWithCode(content: String, codeLeft: Int, codeTop: Int, inverted: Boolean = false, rowPadding: Int = 0): Frame {
		val matrix = QRCodeWriter().encode(content, BarcodeFormat.QR_CODE, CODE_SIZE, CODE_SIZE)
		val dark = if (inverted) WHITE else BLACK
		val light = if (inverted) BLACK else WHITE
		val dataWidth = FRAME_WIDTH + rowPadding
		val data = ByteArray(dataWidth * FRAME_HEIGHT) { light }
		for (y in 0 until matrix.height) {
			for (x in 0 until matrix.width) {
				if (matrix[x, y]) data[(codeTop + y) * dataWidth + codeLeft + x] = dark
			}
		}
		return Frame(data, dataWidth, FRAME_HEIGHT)
	}

	private companion object {
		const val FRAME_WIDTH = 640
		const val FRAME_HEIGHT = 480
		const val CODE_SIZE = 240
		const val BLACK: Byte = 0
		const val WHITE: Byte = -1
	}
}
