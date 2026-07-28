import Foundation
import Testing

/// Anti-drift guard for the CI wiring that publishes `AccountReportTests` output.
///
/// `ReportBuilder` writes its Markdown to `$ACCOUNT_REPORT_PATH` (falling back to
/// repo-root/account-report.md, which equals `$GITHUB_WORKSPACE` on CI). For that
/// report to actually reach a PR, the CI must:
///   1. feed the tests step a workspace-anchored file named `account-report.md`
///      (== `ReportBuilder.fileName`), wired through to `ACCOUNT_REPORT_PATH`
///      (== `ReportBuilder.pathEnvVar`),
///   2. consume that env and surface the file into the job step summary,
///   3. upload the report + the xcresult bundle as an artifact,
///   4. run the test step via the SPM scheme, scoped to the ConnectionTypes target.
///
/// The tests step was extracted out of the workflow into a local composite action
/// (`.github/workflows/apple/tests/action.yml`), so the contract now spans two
/// files: the workflow passes the workspace-anchored path as the action's
/// `account-report-path` input; the action wires it to the env and does everything else
/// (summary, upload, the xcodebuild invocations). These tests read both YAML files off
/// disk and assert that split contract. If either drifts (env renamed/dropped, scheme
/// swapped, upload removed, input handoff broken) — or if the `ReportBuilder` constants
/// change without a matching CI edit — this turns red instead of silently producing no
/// report on CI. (Plain-text scanning, matching the source-reading style of
/// `AccountReportTests.viewButtonTitleKeys`; Foundation ships no YAML parser and the
/// assertions don't need full structure.)
struct WorkflowReportTests {
    private static let workflowRelativePath = ".github/workflows/ci-nym-vpn-app-macos.yml"
    private static let actionRelativePath = ".github/workflows/apple/tests/action.yml"
    /// The composite action's input the workflow passes the report path through.
    private static let reportInputKey = "account-report-path"
    private static let scheme = "ServicesMutual-Package"
    private static let testTarget = "ConnectionTypesTests"
    private static let xcresultBundle = "TestResults.xcresult"

    /// Loads a YAML file, resolving repo root the same way `ReportBuilder` does:
    /// up 5 from this file → repo root (== `$GITHUB_WORKSPACE`).
    private func loadYAML(_ relativePath: String, file: StaticString = #filePath) throws -> String {
        var root = URL(fileURLWithPath: "\(file)")
        for _ in 0..<5 { root.deleteLastPathComponent() }
        let url = root.appendingPathComponent(relativePath)
        return try #require(try? String(contentsOf: url, encoding: .utf8),
                            "YAML not found at \(url.path) — repo layout changed?")
    }

    /// The value assigned to `key` in the YAML (rest of the line after `key:`).
    private func value(after key: String, in yaml: String) throws -> String {
        let regex = try NSRegularExpression(pattern: "\(key):\\s*(.+)")
        let range = NSRange(yaml.startIndex..., in: yaml)
        let match = try #require(regex.firstMatch(in: yaml, range: range),
                                 "\(key) is not set")
        let valueRange = try #require(Range(match.range(at: 1), in: yaml))
        return String(yaml[valueRange]).trimmingCharacters(in: .whitespaces)
    }

    @Test func reportPathEnvWiredToWorkspaceFile() throws {
        // The action wires its ACCOUNT_REPORT_PATH env from the action input…
        let env = try value(after: ReportBuilder.pathEnvVar, in: try loadYAML(Self.actionRelativePath))
        #expect(env.contains("inputs.\(Self.reportInputKey)"),
                "\(ReportBuilder.pathEnvVar) must be wired from the \(Self.reportInputKey) input, got: \(env)")
        // …and the workflow feeds that input a workspace-anchored account-report.md.
        let input = try value(after: Self.reportInputKey, in: try loadYAML(Self.workflowRelativePath))
        #expect(input.contains("github.workspace"),
                "\(Self.reportInputKey) must be anchored at github.workspace, got: \(input)")
        #expect(input.hasSuffix(ReportBuilder.fileName),
                "\(Self.reportInputKey) must end in \(ReportBuilder.fileName), got: \(input)")
    }

    @Test func reportEnvVarIsConsumedAndSurfaced() throws {
        let action = try loadYAML(Self.actionRelativePath)
        // The env must actually be read (not just declared)…
        let consumed = action.contains("$\(ReportBuilder.pathEnvVar)")
            || action.contains("${\(ReportBuilder.pathEnvVar)}")
        #expect(consumed, "\(ReportBuilder.pathEnvVar) is set but never referenced")
        // …and routed into the PR's job step summary.
        #expect(action.contains("GITHUB_STEP_SUMMARY"),
                "report is not surfaced to the job step summary")
    }

    @Test func reportAndResultsUploadedAsArtifact() throws {
        let action = try loadYAML(Self.actionRelativePath)
        // Locate the upload step by its action (rename-proof), then assert both paths.
        // Composite-action steps are indented one level shallower than workflow steps.
        let uploadStep = try #require(
            action.components(separatedBy: "\n    - name:").first { $0.contains("actions/upload-artifact") },
            "no upload-artifact step in the tests action")
        #expect(uploadStep.contains(ReportBuilder.fileName),
                "\(ReportBuilder.fileName) is not in the upload-artifact paths")
        #expect(uploadStep.contains(Self.xcresultBundle),
                "\(Self.xcresultBundle) is not in the upload-artifact paths")
    }

    /// Suites are array-driven: the action loops a `SUITES` array whose entries are
    /// `dir|scheme|target|bundle|label`, so the live invocation is `-scheme "$scheme"`
    /// (no literal scheme in the command). Assert the ConnectionTypes entry is present
    /// as a `|scheme|target|` pair — disambiguates e.g. the "Settings" scheme from the
    /// "Settings" substring that also appears in dirs/paths/labels.
    @Test func testStepRunsConnectionTypes() throws {
        let action = try loadYAML(Self.actionRelativePath)
        #expect(action.contains("|\(Self.scheme)|\(Self.testTarget)|"),
                "tests action has no suite entry for \(Self.scheme) / \(Self.testTarget)")
    }

    /// The Tests job runs more than ConnectionTypes: each lives in its own SwiftPM
    /// package and only runs if its array entry is present. If a suite is dropped from
    /// the action it would pass CI while testing nothing — this turns red instead.
    /// `(scheme, target)` pairs must match the per-package `SUITES` entries.
    @Test(arguments: [
        (scheme: "Services-Package", target: "ConfigurationManagerTests"),
        (scheme: "Settings", target: "SettingsTests"),
        (scheme: "ServicesMacOS-Package", target: "AppDiscoveryServiceTests"),
        (scheme: "Services-Package", target: "CredentialsManagerTests/AppSessionReducerTests"),
        (scheme: "Services-Package", target: "CredentialsManagerTests/AccountPrefetchOrchestratorTests"),
        (scheme: "Home", target: "HomeTests"),
        (scheme: "Services-Package", target: "CredentialsManagerTests/EnvironmentChangeIAPPolicyTests"),
        (scheme: "Services-Package", target: "CredentialsManagerTests/PostPurchaseProcessingPolicyTests"),
        (scheme: "Services-Package", target: "CredentialsManagerTests/PostPurchaseProcessingFlowTests"),
        (scheme: "Services-Package", target: "CredentialsManagerTests/AccountTokenByEnvStorageTests"),
        (scheme: "Services-Package", target: "CredentialsManagerTests/DrawerSessionPolicyTests"),
        (scheme: "Services-Package", target: "CredentialsManagerTests/DrawerCredentialImportPolicyTests"),
    ])
    func testStepRunsAdditionalSuite(scheme: String, target: String) throws {
        let action = try loadYAML(Self.actionRelativePath)
        #expect(action.contains("|\(scheme)|\(target)|"),
                "tests action no longer runs \(target) via the \(scheme) scheme")
    }
}
