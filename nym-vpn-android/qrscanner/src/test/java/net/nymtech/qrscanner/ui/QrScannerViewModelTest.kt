package net.nymtech.qrscanner.ui

import kotlinx.coroutines.CompletableDeferred
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.test.StandardTestDispatcher
import kotlinx.coroutines.test.advanceTimeBy
import kotlinx.coroutines.test.resetMain
import kotlinx.coroutines.test.runCurrent
import kotlinx.coroutines.test.runTest
import kotlinx.coroutines.test.setMain
import net.nymtech.qrscanner.QrScanValidator
import net.nymtech.qrscanner.model.QrScanMode
import net.nymtech.qrscanner.model.QrScanResult
import net.nymtech.qrscanner.model.QrScanValidation
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Before
import org.junit.Test
import kotlin.time.Duration.Companion.seconds
import kotlin.time.TestTimeSource

@OptIn(ExperimentalCoroutinesApi::class)
class QrScannerViewModelTest {
	private val dispatcher = StandardTestDispatcher()
	private val validUrl = "https://nym.com/qr-pairing?pubkey=valid"
	private val otherCode = "WIFI:S:home;T:WPA;P:secret;;"
	private val customMessageRes = 42

	private val validated = mutableListOf<String>()
	private val validator = QrScanValidator { contents ->
		validated += contents
		if (contents == validUrl) QrScanValidation.Valid else QrScanValidation.Invalid()
	}

	@Before
	fun setUp() {
		Dispatchers.setMain(dispatcher)
	}

	@After
	fun tearDown() {
		Dispatchers.resetMain()
	}

	private val timeSource = TestTimeSource()

	private fun viewModel(validator: QrScanValidator = this.validator) = QrScannerViewModel(QrScanMode.PAIRING, validator, MESSAGE_DURATION, timeSource)

	@Test
	fun acceptedCodeBecomesTheResult() = runTest(dispatcher) {
		val viewModel = viewModel()
		viewModel.onCodeScan(validUrl)
		runCurrent()

		assertEquals(QrScanResult.Pairing(validUrl), viewModel.uiState.value.result)
		assertFalse(viewModel.uiState.value.showInvalidCode)
	}

	@Test
	fun rejectedCodeShowsDefaultMessageAndKeepsScanning() = runTest(dispatcher) {
		val viewModel = viewModel()
		viewModel.onCodeScan(otherCode)
		runCurrent()

		val state = viewModel.uiState.value
		assertNull(state.result)
		assertTrue(state.showInvalidCode)
		assertEquals(QrScanMode.PAIRING.invalidCodeRes, state.invalidCodeRes)

		viewModel.onCodeScan(validUrl)
		runCurrent()
		assertEquals(QrScanResult.Pairing(validUrl), viewModel.uiState.value.result)
	}

	@Test
	fun rejectedCodeShowsValidatorMessage() = runTest(dispatcher) {
		val viewModel = viewModel(QrScanValidator { QrScanValidation.Invalid(customMessageRes) })
		viewModel.onCodeScan(otherCode)
		runCurrent()

		assertEquals(customMessageRes, viewModel.uiState.value.invalidCodeRes)
	}

	@Test
	fun rejectedCodeStillInViewIsNotValidatedAgain() = runTest(dispatcher) {
		val viewModel = viewModel()
		repeat(5) {
			viewModel.onCodeScan(otherCode)
			runCurrent()
		}

		assertEquals(listOf(otherCode), validated)
		assertTrue(viewModel.uiState.value.showInvalidCode)
	}

	@Test
	fun codesArrivingDuringValidationAreIgnored() = runTest(dispatcher) {
		val pending = CompletableDeferred<QrScanValidation>()
		val viewModel = viewModel(
			QrScanValidator { contents ->
				validated += contents
				pending.await()
			},
		)
		viewModel.onCodeScan(validUrl)
		runCurrent()
		viewModel.onCodeScan(otherCode)
		runCurrent()
		pending.complete(QrScanValidation.Valid)
		runCurrent()

		assertEquals(listOf(validUrl), validated)
		assertEquals(QrScanResult.Pairing(validUrl), viewModel.uiState.value.result)
	}

	@Test
	fun codesAfterTheResultAreIgnored() = runTest(dispatcher) {
		val viewModel = viewModel()
		viewModel.onCodeScan(validUrl)
		runCurrent()
		viewModel.onCodeScan(otherCode)
		runCurrent()

		assertEquals(listOf(validUrl), validated)
		assertEquals(QrScanResult.Pairing(validUrl), viewModel.uiState.value.result)
	}

	@Test
	fun failingValidatorRejectsTheCode() = runTest(dispatcher) {
		val viewModel = viewModel(QrScanValidator { error("core unavailable") })
		viewModel.onCodeScan(validUrl)
		runCurrent()

		assertNull(viewModel.uiState.value.result)
		assertTrue(viewModel.uiState.value.showInvalidCode)
	}

	@Test
	fun messageHidesOnceTheCodeLeavesView() = runTest(dispatcher) {
		val viewModel = viewModel()
		viewModel.onCodeScan(otherCode)
		runCurrent()
		advanceTimeBy(MESSAGE_DURATION - 1.seconds / 2)
		viewModel.onCodeScan(otherCode)
		runCurrent()
		advanceTimeBy(MESSAGE_DURATION - 1.seconds / 2)
		assertTrue(viewModel.uiState.value.showInvalidCode)

		advanceTimeBy(1.seconds)
		assertFalse(viewModel.uiState.value.showInvalidCode)
	}

	@Test
	fun temporarilyRejectedCodeIsRetriedAfterTheDelay() = runTest(dispatcher) {
		var offline = true
		val viewModel = viewModel(
			QrScanValidator { contents ->
				validated += contents
				if (offline) QrScanValidation.Invalid(customMessageRes, retryAfter = RETRY_AFTER) else QrScanValidation.Valid
			},
		)
		viewModel.onCodeScan(validUrl)
		runCurrent()
		offline = false
		timeSource += RETRY_AFTER / 2
		viewModel.onCodeScan(validUrl)
		runCurrent()
		assertEquals(listOf(validUrl), validated)
		assertEquals(customMessageRes, viewModel.uiState.value.invalidCodeRes)

		timeSource += RETRY_AFTER
		viewModel.onCodeScan(validUrl)
		runCurrent()
		assertEquals(listOf(validUrl, validUrl), validated)
		assertEquals(QrScanResult.Pairing(validUrl), viewModel.uiState.value.result)
	}

	@Test
	fun fallbackBecomesTheResult() = runTest(dispatcher) {
		val viewModel = viewModel()
		viewModel.onFallback()
		viewModel.onCodeScan(validUrl)
		runCurrent()

		assertEquals(QrScanResult.Fallback, viewModel.uiState.value.result)
		assertTrue(validated.isEmpty())
	}

	@Test
	fun fallbackAfterTheResultIsIgnored() = runTest(dispatcher) {
		val viewModel = viewModel()
		viewModel.onCodeScan(validUrl)
		runCurrent()
		viewModel.onFallback()

		assertEquals(QrScanResult.Pairing(validUrl), viewModel.uiState.value.result)
	}

	private companion object {
		val RETRY_AFTER = 3.seconds
		val MESSAGE_DURATION = 2.seconds
	}
}
