import Foundation

/// Accumulates Markdown table rows across tests and writes them once.
/// Rows are appended BEFORE assertions so the file is complete even if an
/// assertion fails. Flushed from `AccountReportTests.tearDown` (class-level).
///
/// Cells are stored per-column so `markdown()` can pad every column to its
/// widest cell — the raw `.md` then aligns in a monospace view.
final class ReportBuilder {
    static let shared = ReportBuilder()

    /// Single source of truth for the report file name + env override. Shared with
    /// `WorkflowReportTests`, which asserts the CI workflow wires these exact values —
    /// so a rename here (or in the workflow) can't silently break report publishing.
    static let fileName = "account-report.md"
    static let pathEnvVar = "ACCOUNT_REPORT_PATH"

    private static let textHeader = [
        "Plan", "Days left", "Active", "Expiring soon",
        "Expiring warning", "Branch key", "Displayed text", "Color"
    ]
    private static let buttonHeader = ["Scenario", "Actions"]

    private(set) var textRows: [[String]] = []
    private(set) var buttonRows: [[String]] = []

    func addTextRow(_ cells: [String]) { textRows.append(cells) }
    func addButtonRow(_ cells: [String]) { buttonRows.append(cells) }

    func markdown() -> String {
        var out = "# Account & Devices — test report\n\n"
        out += "## Text by scenario (macOS)\n\n"
        out += Self.renderTable(header: Self.textHeader, rows: textRows)
        out += "\n## Buttons & actions by scenario\n\n"
        out += Self.renderTable(header: Self.buttonHeader, rows: buttonRows)
        return out
    }

    /// GitHub-flavored Markdown table with each column padded to its widest cell
    /// so the raw text lines up. (Emoji may render slightly wider than their
    /// character count, but every column is consistently character-aligned.)
    private static func renderTable(header: [String], rows: [[String]]) -> String {
        let columnCount = header.count
        var widths = header.map { $0.count }
        for row in rows {
            for (i, cell) in row.enumerated() where i < columnCount {
                widths[i] = max(widths[i], cell.count)
            }
        }

        func line(_ cells: [String]) -> String {
            let padded = (0..<columnCount).map { i -> String in
                let cell = i < cells.count ? cells[i] : ""
                return cell.padding(toLength: widths[i], withPad: " ", startingAt: 0)
            }
            return "| " + padded.joined(separator: " | ") + " |\n"
        }

        var out = line(header)
        out += "| " + widths.map { String(repeating: "-", count: $0) }.joined(separator: " | ") + " |\n"
        for row in rows { out += line(row) }
        return out
    }

    func write(file: StaticString = #filePath) {
        let envPath = ProcessInfo.processInfo.environment[Self.pathEnvVar].flatMap { $0.isEmpty ? nil : $0 }
        let path = envPath ?? Self.defaultReportPath(file: file)
        try? markdown().write(toFile: path, atomically: true, encoding: .utf8)
        FileHandle.standardError.write(Data("account-report written to \(path)\n".utf8))
    }

    /// repo-root/account-report.md, derived from this test file's on-disk location:
    /// …/nym-vpn-client/nym-vpn-apple/ServicesMutual/Tests/ConnectionTypesTests/<file>.swift
    /// → up 5 → repo root (nym-vpn-client), which equals CI's $GITHUB_WORKSPACE.
    private static func defaultReportPath(file: StaticString) -> String {
        var url = URL(fileURLWithPath: "\(file)")
        for _ in 0..<5 { url.deleteLastPathComponent() }
        return url.appendingPathComponent(Self.fileName).path
    }
}
