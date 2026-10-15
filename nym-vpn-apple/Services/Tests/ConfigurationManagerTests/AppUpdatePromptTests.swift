import Testing
@testable import ConfigurationManager

struct AppUpdatePromptTests {
    @Test func belowTheFloorOnlyRequiredBlocksConnect() {
        func prompt(_ policy: String) -> AppUpdatePrompt {
            appUpdatePrompt(local: "2026.10.0", floor: "2026.12.4", policy: policy)
        }
        #expect(prompt("required") == AppUpdatePrompt(show: true, blockConnect: true))
        #expect(prompt("dismissible") == AppUpdatePrompt(show: true, blockConnect: false))
        #expect(prompt("nope") == AppUpdatePrompt(show: true, blockConnect: false))
    }

    @Test func atTheFloorOrWithoutVersionsDoesNotPrompt() {
        let none = AppUpdatePrompt(show: false, blockConnect: false)
        #expect(appUpdatePrompt(local: "2026.12.4", floor: "2026.12.4", policy: "required") == none)
        #expect(appUpdatePrompt(local: nil, floor: "2026.12.4", policy: "required") == none)
        #expect(appUpdatePrompt(local: "2026.10.0", floor: "", policy: "required") == none)
    }

    @Test func aPlatformPolicyBeatsTheDefaultAndAPrereleaseIsBelowTheFloor() {
        let override = appUpdatePrompt(
            local: "2026.10.0",
            floor: "2026.12.4",
            policy: "dismissible",
            policies: ["ios": "required"],
            platform: "ios"
        )
        #expect(override == AppUpdatePrompt(show: true, blockConnect: true))
        let beta = appUpdatePrompt(local: "2026.13.0-beta.1", floor: "2026.13.0", policy: "dismissible")
        #expect(beta == AppUpdatePrompt(show: true, blockConnect: false))
    }
}
