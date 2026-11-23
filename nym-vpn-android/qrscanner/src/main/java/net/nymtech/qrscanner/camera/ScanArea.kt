package net.nymtech.qrscanner.camera

// Shared by the overlay and the analyzer so both agree on where codes are read.
internal object ScanArea {
	const val WINDOW_FRACTION = 0.7f

	// Slightly larger than the window so off-centre codes still read.
	const val ANALYSIS_FRACTION = 0.85f
}
