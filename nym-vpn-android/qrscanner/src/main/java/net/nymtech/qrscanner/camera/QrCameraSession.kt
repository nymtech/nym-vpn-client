package net.nymtech.qrscanner.camera

import android.content.Context
import android.util.Size
import androidx.camera.core.Camera
import androidx.camera.core.CameraSelector
import androidx.camera.core.ImageAnalysis
import androidx.camera.core.Preview
import androidx.camera.core.UseCaseGroup
import androidx.camera.core.resolutionselector.ResolutionSelector
import androidx.camera.core.resolutionselector.ResolutionStrategy
import androidx.camera.lifecycle.ProcessCameraProvider
import androidx.camera.view.PreviewView
import androidx.core.content.ContextCompat
import androidx.core.view.doOnLayout
import androidx.lifecycle.LifecycleOwner
import net.nymtech.qrscanner.decoder.QrDecoder
import net.nymtech.qrscanner.decoder.ZxingQrDecoder
import timber.log.Timber
import java.util.concurrent.Executors

// Single-use: preview and analysis only, never capture. Create and call on the main thread.
internal class QrCameraSession(
	context: Context,
	private val lifecycleOwner: LifecycleOwner,
	private val previewView: PreviewView,
	private val listener: Listener,
	decoder: QrDecoder = ZxingQrDecoder(),
) {
	interface Listener {
		fun onCodeScan(text: String)

		fun onTorchAvailabilityChange(available: Boolean)

		fun onCameraError()
	}

	private val mainExecutor = ContextCompat.getMainExecutor(context)
	private val providerFuture = ProcessCameraProvider.getInstance(context)
	private val analysisExecutor = Executors.newSingleThreadExecutor()
	private val analyzer = QrCodeAnalyzer(decoder) { text ->
		mainExecutor.execute { if (!released) listener.onCodeScan(text) }
	}
	private val preview = Preview.Builder().build()
	private val analysis = ImageAnalysis.Builder()
		.setBackpressureStrategy(ImageAnalysis.STRATEGY_KEEP_ONLY_LATEST)
		.setResolutionSelector(
			ResolutionSelector.Builder()
				.setResolutionStrategy(ResolutionStrategy(ANALYSIS_RESOLUTION, ResolutionStrategy.FALLBACK_RULE_CLOSEST_HIGHER_THEN_LOWER))
				.build(),
		)
		.build()

	private var provider: ProcessCameraProvider? = null
	private var camera: Camera? = null
	private var released = false

	fun start() {
		providerFuture.addListener({
			if (released) return@addListener
			try {
				val cameraProvider = providerFuture.get()
				provider = cameraProvider
				// The viewport needs the view's size.
				previewView.doOnLayout { if (!released) bind(cameraProvider) }
			} catch (e: Exception) {
				fail(e)
			}
		}, mainExecutor)
	}

	fun setTorchEnabled(enabled: Boolean) {
		camera?.takeIf { it.cameraInfo.hasFlashUnit() }?.cameraControl?.enableTorch(enabled)
	}

	fun stopScanning() {
		analyzer.stop()
		analysis.clearAnalyzer()
	}

	fun release() {
		if (released) return
		released = true
		stopScanning()
		setTorchEnabled(false)
		provider?.unbind(preview, analysis)
		camera = null
		// Queued behind any in-flight frame so the buffer is wiped after its last use.
		analysisExecutor.execute(analyzer::wipe)
		analysisExecutor.shutdown()
	}

	private fun bind(cameraProvider: ProcessCameraProvider) {
		try {
			preview.setSurfaceProvider(previewView.surfaceProvider)
			analysis.setAnalyzer(analysisExecutor, analyzer)
			val useCases = UseCaseGroup.Builder()
				.addUseCase(preview)
				.addUseCase(analysis)
				.apply { previewView.viewPort?.let { setViewPort(it) } }
				.build()
			val boundCamera = cameraProvider.bindToLifecycle(lifecycleOwner, cameraProvider.preferredCamera(), useCases)
			camera = boundCamera
			listener.onTorchAvailabilityChange(boundCamera.cameraInfo.hasFlashUnit())
		} catch (e: Exception) {
			fail(e)
		}
	}

	// Front camera as a fallback for Chromebooks.
	private fun ProcessCameraProvider.preferredCamera(): CameraSelector = if (hasCamera(CameraSelector.DEFAULT_BACK_CAMERA)) CameraSelector.DEFAULT_BACK_CAMERA else CameraSelector.DEFAULT_FRONT_CAMERA

	private fun fail(error: Exception) {
		Timber.w(error, "QR scanner camera failed to start")
		if (!released) listener.onCameraError()
	}

	private companion object {
		val ANALYSIS_RESOLUTION = Size(1280, 720)
	}
}
