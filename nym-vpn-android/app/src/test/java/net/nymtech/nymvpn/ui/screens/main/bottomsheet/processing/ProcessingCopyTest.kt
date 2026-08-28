package net.nymtech.nymvpn.ui.screens.main.bottomsheet.processing

import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.AuthRoute
import net.nymtech.nymvpn.ui.Route
import net.nymtech.nymvpn.ui.routeName
import nym_vpn_lib_types.AccountControllerErrorStateReason
import nym_vpn_lib_types.AccountControllerState
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test

class ProcessingCopyTest {

	@Test
	fun inactiveRoutesToSelectPlan() {
		val pending = ProcessingCopy.postLoginRoute(
			AccountControllerState.PendingSubscription,
			showTechnicalOpt = false,
		)
		assertEquals(Route.SelectPlan, pending)

		val inactive = ProcessingCopy.postLoginRoute(
			AccountControllerState.Error(
				AccountControllerErrorStateReason.InactiveSubscription,
			),
			showTechnicalOpt = true,
		)
		assertEquals(Route.SelectPlan, inactive)
	}

	@Test
	fun activeRoutesToMainOrTechOpt() {
		val main = ProcessingCopy.postLoginRoute(
			AccountControllerState.ReadyToConnect,
			showTechnicalOpt = false,
		)
		assertEquals(Route.Main(), main)

		val techOpt = ProcessingCopy.postLoginRoute(
			AccountControllerState.ReadyToConnect,
			showTechnicalOpt = true,
		)
		assertEquals(Route.Main(authRoute = AuthRoute.TechOpt.routeName), techOpt)
	}

	@Test
	fun timeoutProceedsToMain() {
		val main = ProcessingCopy.timeoutRoute(showTechnicalOpt = false)
		assertEquals(Route.Main(), main)

		val techOpt = ProcessingCopy.timeoutRoute(showTechnicalOpt = true)
		assertEquals(Route.Main(authRoute = AuthRoute.TechOpt.routeName), techOpt)
	}

	@Test
	fun loginProgressStepForCarouselIndex_matchesIosSteps() {
		assertEquals(2, ProcessingCopy.loginProgressStepForCarouselIndex(0))
		assertEquals(3, ProcessingCopy.loginProgressStepForCarouselIndex(1))
		assertEquals(4, ProcessingCopy.loginProgressStepForCarouselIndex(2))
	}

	@Test
	fun setupCarouselPairs_useDistinctSubtitles() {
		val subtitles = (0 until ProcessingCopy.CAROUSEL_TICK_COUNT).map { index ->
			ProcessingCopy.setupCarouselPairRes(index).subtitleRes
		}
		assertEquals(subtitles.toSet().size, subtitles.size)
		assertTrue(subtitles.all { it != null })
	}

	@Test
	fun setupCarouselTextLeadsStepBar_byConfiguredDelay() {
		assertTrue(ProcessingCopy.textAdvancePrecedesStepBarTick())
	}

	@Test
	fun credentialsStepTwoCopy_doesNotPromiseConnect() {
		val combined = listOf(
			"Finishing up…",
			"Just a few more seconds",
		).joinToString(" ").lowercase()
		ProcessingCopy.CREDENTIALS_STEP_TWO_FORBIDDEN_TERMS.forEach { term ->
			assertFalse("Credentials step 2 must not contain $term", combined.contains(term))
		}
	}

	@Test
	fun credentialsCarouselPairRes_tick0_and_tick1() {
		assertEquals(
			R.string.account_login_processing_loading_credentials to
				R.string.account_login_processing_loading_credentials_subtitle,
			ProcessingCopy.credentialsCarouselPairRes(0),
		)
		assertEquals(
			R.string.account_login_processing_almost_ready to
				R.string.account_login_processing_almost_ready_subtitle,
			ProcessingCopy.credentialsCarouselPairRes(1),
		)
		assertEquals(
			R.string.account_login_processing_almost_ready to
				R.string.account_login_processing_almost_ready_subtitle,
			ProcessingCopy.credentialsCarouselPairRes(99),
		)
	}

	@Test
	fun processingCopyForPhase_credentialsCarousel_includesSubtitles() {
		val tick0 = ProcessingCopy.processingCopyForPhase(
			LoginProcessingUiPhase.Carousel,
			AccountControllerState.Syncing,
			credentialsCarouselTick = 0,
			setupCarouselFinished = true,
		)
		assertEquals(R.string.account_login_processing_loading_credentials, tick0.titleRes)
		assertEquals(R.string.account_login_processing_loading_credentials_subtitle, tick0.subtitleRes)

		val tick1 = ProcessingCopy.processingCopyForPhase(
			LoginProcessingUiPhase.Carousel,
			AccountControllerState.Syncing,
			credentialsCarouselTick = 1,
			setupCarouselFinished = true,
		)
		assertEquals(R.string.account_login_processing_almost_ready, tick1.titleRes)
		assertEquals(R.string.account_login_processing_almost_ready_subtitle, tick1.subtitleRes)
	}

	@Test
	fun processingCopyForPhase_syncingAfterSetup_showsCredentialsCopy() {
		val copy = ProcessingCopy.processingCopyForPhase(
			LoginProcessingUiPhase.Carousel,
			AccountControllerState.Syncing,
			credentialsCarouselTick = 0,
			setupCarouselFinished = true,
		)
		assertEquals(R.string.account_login_processing_loading_credentials, copy.titleRes)
		assertEquals(R.string.account_login_processing_loading_credentials_subtitle, copy.subtitleRes)
	}

	@Test
	fun processingCopyForPhase_setupCarousel_showsDistinctSubtitles() {
		val step0 = ProcessingCopy.processingCopyForPhase(
			LoginProcessingUiPhase.Carousel,
			accountState = null,
			setupCarouselIndex = 0,
			setupCarouselFinished = false,
		)
		assertEquals(R.string.account_login_processing_setting_up_step2_subtitle, step0.subtitleRes)

		val step2 = ProcessingCopy.processingCopyForPhase(
			LoginProcessingUiPhase.Carousel,
			accountState = null,
			setupCarouselIndex = 2,
			setupCarouselFinished = false,
		)
		assertEquals(R.string.account_login_processing_setting_up_step4_subtitle, step2.subtitleRes)
	}

	@Test
	fun processingCopyForPhase_syncingBeforeSetup_keepsSetupCopy() {
		val copy = ProcessingCopy.processingCopyForPhase(
			LoginProcessingUiPhase.Carousel,
			AccountControllerState.Syncing,
			credentialsCarouselTick = 1,
			setupCarouselFinished = false,
		)
		assertEquals(R.string.account_login_processing_setting_up, copy.titleRes)
		assertEquals(R.string.account_login_processing_setting_up_step2_subtitle, copy.subtitleRes)
	}

	@Test
	fun processingTitleForPhase() {
		assertEquals(
			R.string.account_login_processing_setting_up,
			ProcessingCopy.processingTitleForPhase(LoginProcessingUiPhase.Carousel),
		)
		assertEquals(
			R.string.account_login_processing_loading_credentials,
			ProcessingCopy.processingCopyForPhase(
				LoginProcessingUiPhase.Carousel,
				AccountControllerState.Syncing,
				setupCarouselFinished = true,
			).titleRes,
		)
		assertEquals(
			R.string.account_payment_welcome,
			ProcessingCopy.processingTitleForPhase(LoginProcessingUiPhase.Welcome),
		)
	}

	@Test
	fun processingTitleForCarouselAccountState() {
		assertEquals(
			R.string.account_login_processing_setting_up,
			ProcessingCopy.processingTitleForCarouselAccountState(AccountControllerState.ReadyToConnect),
		)
		assertEquals(
			R.string.account_login_processing_loading_credentials,
			ProcessingCopy.processingTitleForCarouselAccountState(AccountControllerState.Syncing),
		)
	}

	@Test
	fun failureMessageResForState_maxDeviceReached() {
		val res = ProcessingCopy.failureMessageResForState(
			AccountControllerState.Error(AccountControllerErrorStateReason.MaxDeviceReached),
		)
		assertEquals(R.string.max_devices_reached_title, res)
	}

	@Test
	fun failureMessageResForState_apiFailure_usesGenericError() {
		val res = ProcessingCopy.failureMessageResForState(
			AccountControllerState.Error(
				AccountControllerErrorStateReason.ApiFailure(context = "login", details = "timeout"),
			),
		)
		assertEquals(R.string.account_generating_error, res)
	}

	@Test
	fun carouselDurationMs_matchesSlowerSetupCarousel() {
		assertEquals(10_000L, ProcessingCopy.carouselDurationMs())
	}
}
