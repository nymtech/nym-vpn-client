import Foundation
import SwiftUI
import Testing
import AccountPrefetchGates
@testable import Home

@MainActor
struct ProcessingAccountCarouselInterruptTests {
    @Test func workCompleteThenInterrupted_advancesWithoutAnimationCallback() async {
        let processing = FakeProcessing()
        let coordinator = FakeCoordinator()
        let viewModel = makeViewModel(flow: .login, processing: processing, coordinator: coordinator)

        await viewModel.run()
        #expect(viewModel.phase == .awaitingAdvance)
        #expect(!viewModel.didFinishSetupCarousel)
        #expect(coordinator.actions.isEmpty)

        viewModel.noteCarouselInterrupted()
        await viewModel.awaitFinalMessage()

        #expect(viewModel.didFinishSetupCarousel)
        #expect(viewModel.phase == .finished)
        #expect(coordinator.actions == [.session(.processingFinished)])
    }

    @Test func interruptedBeforeWorkCompletes_latchesWhenWorkFinishes() async {
        let processing = FakeProcessing()
        let coordinator = FakeCoordinator()
        let viewModel = makeViewModel(flow: .createAccount, processing: processing, coordinator: coordinator)

        viewModel.noteCarouselInterrupted()
        #expect(!viewModel.didFinishSetupCarousel)
        #expect(coordinator.actions.isEmpty)

        await viewModel.run()
        await viewModel.awaitFinalMessage()

        #expect(viewModel.didFinishSetupCarousel)
        #expect(viewModel.phase == .finished)
        #expect(coordinator.actions == [.session(.processingFinished)])
    }

    @Test func resumedBeforeWorkCompletes_stillWaitsForAnimation() async {
        let processing = FakeProcessing()
        let coordinator = FakeCoordinator()
        let viewModel = makeViewModel(flow: .login, processing: processing, coordinator: coordinator)

        viewModel.noteCarouselInterrupted()
        viewModel.noteCarouselResumed()
        await viewModel.run()

        #expect(!viewModel.didFinishSetupCarousel)
        #expect(!viewModel.didFinishAnimatingText)
        #expect(viewModel.phase == .awaitingAdvance)
        #expect(viewModel.currentStep == LoginProcessingUI.initialProgressStep)
        #expect(coordinator.actions.isEmpty)
    }

    @Test func interruptDuringInFlightPrepare_doesNotJumpBarsUntilWorkCompletes() async {
        let processing = FakeProcessing()
        processing.holdPrepareUntilReleased = true
        let coordinator = FakeCoordinator()
        let viewModel = makeViewModel(flow: .login, processing: processing, coordinator: coordinator)

        let runTask = Task { await viewModel.run() }
        await Task.yield()
        try? await Task.sleep(for: .milliseconds(20))
        viewModel.noteCarouselInterrupted()

        #expect(!viewModel.didFinishSetupCarousel)
        #expect(viewModel.currentStep == LoginProcessingUI.initialProgressStep)
        #expect(coordinator.actions.isEmpty)

        processing.releasePrepare()
        await runTask.value
        await viewModel.awaitFinalMessage()
        #expect(viewModel.phase == .finished)
        #expect(coordinator.actions == [.session(.processingFinished)])
    }

    @Test func scenePhaseMapsOnlyBackgroundToInterrupt() {
        #expect(ProcessingAccountView.carouselSceneAction(for: .active) == .resume)
        #expect(ProcessingAccountView.carouselSceneAction(for: .inactive) == .ignore)
        #expect(ProcessingAccountView.carouselSceneAction(for: .background) == .interrupt)
    }
}
