import Testing
@testable import ConfigurationManager

struct AppUpdatePromptTests {
    @Test func dismissibleBelowFloorShowsAndAllowsConnect() {
        let decision = appUpdatePrompt(local: "2026.10.0", floor: "2026.12.4", policy: "dismissible")
        #expect(decision == AppUpdatePrompt(show: true, blockConnect: false))
    }

    @Test func requiredBelowFloorShowsAndBlocksConnect() {
        let decision = appUpdatePrompt(local: "2026.10.0", floor: "2026.12.4", policy: "required")
        #expect(decision == AppUpdatePrompt(show: true, blockConnect: true))
    }

    @Test func equalFloorDoesNotPrompt() {
        let decision = appUpdatePrompt(local: "2026.12.4", floor: "2026.12.4", policy: "required")
        #expect(decision == AppUpdatePrompt(show: false, blockConnect: false))
    }

    @Test func missingVersionsAllowConnect() {
        let decision = appUpdatePrompt(local: nil, floor: nil, policy: "required")
        #expect(decision == AppUpdatePrompt(show: false, blockConnect: false))
    }

    @Test func oldMarketingVersionIsBelowCalendarFloor() {
        let decision = appUpdatePrompt(local: "2.15.1", floor: "2026.12.4", policy: "dismissible")
        #expect(decision.show)
    }
}
