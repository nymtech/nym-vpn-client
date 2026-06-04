import Foundation
import Testing

/// Anti-drift guard for the CI wiring that publishes `AccountReportTests` output.
///
/// `ReportBuilder` writes its Markdown to `$ACCOUNT_REPORT_PATH` (falling back to
/// repo-root/account-report.md, which equals `$GITHUB_WORKSPACE` on CI). For that
/// report to actually reach a PR, the GitHub Actions workflow must:
///   1. set `ACCOUNT_REPORT_PATH` (== `ReportBuilder.pathEnvVar`) to a workspace-anchored
///      file named `account-report.md` (== `ReportBuilder.fileName`),
///   2. consume that env and surface the file into the job step summary,
///   3. upload the report + the xcresult bundle as an artifact,
///   4. run the test step via the SPM scheme, scoped to the ConnectionTypes target.
///
/// These tests read the workflow YAML off disk and assert that contract. If the
/// workflow drifts (env renamed/dropped, scheme swapped, upload removed) — or if the
/// `ReportBuilder` constants change without a matching workflow edit — this turns red
/// instead of silently producing no report on CI. (Plain-text scanning, matching the
/// source-reading style of `AccountReportTests.viewButtonTitleKeys`; Foundation ships
/// no YAML parser and the assertions don't need full structure.)
struct WorkflowReportTests {
    private static let workflowRelativePath = ".github/workflows/build-nym-vpn-apple.yml"
    private static let scheme = "ServicesMutual-Package"
    private static let testTarget = "ConnectionTypesTests"
    private static let xcresultBundle = "TestResults.xcresult"

    /// Loads the workflow YAML, resolving repo root the same way `ReportBuilder` does:
    /// up 5 from this file → repo root (== `$GITHUB_WORKSPACE`).
    private func workflowYAML(file: StaticString = #filePath) throws -> String {
        var root = URL(fileURLWithPath: "\(file)")
        for _ in 0..<5 { root.deleteLastPathComponent() }
        let url = root.appendingPathComponent(Self.workflowRelativePath)
        return try #require(try? String(contentsOf: url, encoding: .utf8),
                            "workflow not found at \(url.path) — repo layout changed?")
    }

    /// The value the workflow assigns to `ReportBuilder.pathEnvVar` (rest of the line).
    private func reportPathEnvValue(in yaml: String) throws -> String {
        let regex = try NSRegularExpression(pattern: "\(ReportBuilder.pathEnvVar):\\s*(.+)")
        let range = NSRange(yaml.startIndex..., in: yaml)
        let match = try #require(regex.firstMatch(in: yaml, range: range),
                                 "\(ReportBuilder.pathEnvVar) is not set in the workflow")
        let valueRange = try #require(Range(match.range(at: 1), in: yaml))
        return String(yaml[valueRange]).trimmingCharacters(in: .whitespaces)
    }

    @Test func reportPathEnvWiredToWorkspaceFile() throws {
        let value = try reportPathEnvValue(in: try workflowYAML())
        // Anchored at the runner workspace (== ReportBuilder's up-5 fallback root)…
        #expect(value.contains("github.workspace"),
                "\(ReportBuilder.pathEnvVar) must be anchored at github.workspace, got: \(value)")
        // …and named exactly what ReportBuilder writes.
        #expect(value.hasSuffix(ReportBuilder.fileName),
                "\(ReportBuilder.pathEnvVar) must end in \(ReportBuilder.fileName), got: \(value)")
    }

    @Test func reportEnvVarIsConsumedAndSurfaced() throws {
        let yaml = try workflowYAML()
        // The env must actually be read (not just declared)…
        let consumed = yaml.contains("$\(ReportBuilder.pathEnvVar)")
            || yaml.contains("${\(ReportBuilder.pathEnvVar)}")
        #expect(consumed, "\(ReportBuilder.pathEnvVar) is set but never referenced")
        // …and routed into the PR's job step summary.
        #expect(yaml.contains("GITHUB_STEP_SUMMARY"),
                "report is not surfaced to the job step summary")
    }

    @Test func reportAndResultsUploadedAsArtifact() throws {
        let yaml = try workflowYAML()
        // Locate the upload step by its action (rename-proof), then assert both paths.
        let uploadStep = try #require(
            yaml.components(separatedBy: "\n      - name:").first { $0.contains("actions/upload-artifact") },
            "no upload-artifact step in the workflow")
        #expect(uploadStep.contains(ReportBuilder.fileName),
                "\(ReportBuilder.fileName) is not in the upload-artifact paths")
        #expect(uploadStep.contains(Self.xcresultBundle),
                "\(Self.xcresultBundle) is not in the upload-artifact paths")
    }

    @Test func testStepRunsConnectionTypes() throws {
        let yaml = try workflowYAML()
        #expect(yaml.contains("-scheme \(Self.scheme)"),
                "test step does not use scheme \(Self.scheme)")
        #expect(yaml.contains("-only-testing:\(Self.testTarget)"),
                "test step does not scope to \(Self.testTarget)")
    }
}
