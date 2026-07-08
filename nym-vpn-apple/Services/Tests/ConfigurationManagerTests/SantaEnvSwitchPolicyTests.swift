#if SANTA
import Foundation
import Testing
@testable import ConfigurationManager

struct SantaEnvSwitchPolicyTests {
    private struct Case: Sendable {
        let label: String
        let isSantaBuild: Bool
        let isTestFlight: Bool
        let isMacOS: Bool
        let isRunningOnCI: Bool
        let isDebugBuild: Bool
        let expected: Bool
    }

    private static let cases: [Case] = [
        Case(label: "nonSanta", isSantaBuild: false, isTestFlight: true, isMacOS: true, isRunningOnCI: true, isDebugBuild: true, expected: false),
        Case(label: "santaDebug", isSantaBuild: true, isTestFlight: false, isMacOS: false, isRunningOnCI: false, isDebugBuild: true, expected: true),
        Case(label: "santaTestFlight", isSantaBuild: true, isTestFlight: true, isMacOS: false, isRunningOnCI: false, isDebugBuild: false, expected: true),
        Case(label: "santaMacOS", isSantaBuild: true, isTestFlight: false, isMacOS: true, isRunningOnCI: false, isDebugBuild: false, expected: true),
        Case(label: "santaCI", isSantaBuild: true, isTestFlight: false, isMacOS: false, isRunningOnCI: true, isDebugBuild: false, expected: true),
        Case(label: "santaIOSRelease", isSantaBuild: true, isTestFlight: false, isMacOS: false, isRunningOnCI: false, isDebugBuild: false, expected: false),
    ]

    @Test(arguments: Self.cases)
    func runtimeAccessGate(_ testCase: Case) {
        #expect(
            SantaEnvSwitchPolicy.canApplyEnvironmentChange(
                isSantaBuild: testCase.isSantaBuild,
                isTestFlight: testCase.isTestFlight,
                isMacOS: testCase.isMacOS,
                isRunningOnCI: testCase.isRunningOnCI,
                isDebugBuild: testCase.isDebugBuild
            ) == testCase.expected,
            Comment(rawValue: testCase.label)
        )
    }
}
#endif
