import Testing
import AppUpdatePrompt

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
        #expect(appUpdatePrompt(local: "2026.10.0", floor: "", policy: "required") == decision)
    }

    @Test func unknownPolicyIsDismissible() {
        let decision = appUpdatePrompt(local: "2026.10.0", floor: "2026.12.4", policy: "nope")
        #expect(decision == AppUpdatePrompt(show: true, blockConnect: false))
    }

    @Test func prereleaseOfTheFloorVersionIsBelowTheFloor() {
        let decision = appUpdatePrompt(local: "2026.13.0-beta.1", floor: "2026.13.0", policy: "dismissible")
        #expect(decision == AppUpdatePrompt(show: true, blockConnect: false))
    }
}
