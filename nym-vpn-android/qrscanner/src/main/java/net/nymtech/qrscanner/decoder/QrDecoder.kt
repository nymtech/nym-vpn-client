package net.nymtech.qrscanner.decoder

internal fun interface QrDecoder {
	fun decode(frame: LuminanceFrame): String?
}

// dataWidth includes any row padding.
internal class LuminanceFrame(val data: ByteArray, val dataWidth: Int, val dataHeight: Int, val left: Int, val top: Int, val width: Int, val height: Int)
