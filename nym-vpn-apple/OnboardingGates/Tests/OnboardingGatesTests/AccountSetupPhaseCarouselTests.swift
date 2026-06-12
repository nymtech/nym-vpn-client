import XCTest
import OnboardingGates

final class AccountSetupPhaseCarouselTests: XCTestCase {
    func testPostPurchaseFetchingTicketsUsesFourthStep() {
        XCTAssertEqual(AccountSetupPhase.carouselStep(for: .fetchingTickets, postPurchase: true), 4)
    }

    func testPrePurchaseFetchingTicketsUsesThirdStep() {
        XCTAssertEqual(AccountSetupPhase.carouselStep(for: .fetchingTickets, postPurchase: false), 3)
    }

    func testIdlePhaseDoesNotAdvanceCarousel() {
        XCTAssertNil(AccountSetupPhase.carouselStep(for: .idle, postPurchase: true))
    }
}
