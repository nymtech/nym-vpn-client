import Foundation
import Testing
@testable import ConnectionTypes

/// `.serialized`: the two row-producing tests append to the shared `ReportBuilder`
/// singleton and each flushes it at the end. Serializing keeps those appends from
/// interleaving (Swift Testing runs tests in parallel by default) and makes the final
/// on-disk report deterministic — `markdown()` renders the full accumulated state, so
/// whichever row-test runs last writes the complete file. (Replaces the old XCTest
/// class-level `tearDown` flush, which has no Swift Testing equivalent.)
@Suite(.serialized)
struct AccountReportTests {
    struct Scenario {
        let planLabel: String
        let daysRemaining: Int?
        let kind: VpnSubscriptionKind
    }

    // Mirrors SantasViewModel.yearlyPresets + monthlyPresets.
    static let scenarios: [Scenario] = [
        .init(planLabel: "Yearly", daysRemaining: 182, kind: .oneYear),
        .init(planLabel: "Yearly", daysRemaining: 61, kind: .oneYear),
        .init(planLabel: "Yearly", daysRemaining: 30, kind: .oneYear),
        .init(planLabel: "Yearly", daysRemaining: 14, kind: .oneYear),
        .init(planLabel: "Yearly", daysRemaining: 7, kind: .oneYear),
        .init(planLabel: "Yearly", daysRemaining: 3, kind: .oneYear),
        .init(planLabel: "Yearly", daysRemaining: 1, kind: .oneYear),
        .init(planLabel: "Yearly", daysRemaining: nil, kind: .oneYear),
        .init(planLabel: "Monthly", daysRemaining: 30, kind: .oneMonth),
        .init(planLabel: "Monthly", daysRemaining: 14, kind: .oneMonth),
        .init(planLabel: "Monthly", daysRemaining: 6, kind: .oneMonth),
        .init(planLabel: "Monthly", daysRemaining: 3, kind: .oneMonth),
        .init(planLabel: "Monthly", daysRemaining: 1, kind: .oneMonth),
        .init(planLabel: "Monthly", daysRemaining: nil, kind: .oneMonth)
    ]

    /// Resolved once; English (source-language) xcstrings values.
    static let resolver: XCStringsResolver? = try? XCStringsResolver.default()

    /// The branch key the production property selects, under the test bundle (raw keys).
    private func expectedBranchKey(_ s: AccountSummary) -> String {
        if !s.isActive { return "noActivePlan" }
        return (s.isExpiringSoon || s.isExpiringWarning) ? "planExpiresOn" : "planValidUntil"
    }

    /// Real displayed sentence, rebuilt from the resolved English strings.
    private func displayedText(_ s: AccountSummary, _ r: XCStringsResolver) -> String {
        if !s.isActive { return r.string("noActivePlan") }
        let date = s.formattedValidUntilDate ?? "-"
        let key = (s.isExpiringSoon || s.isExpiringWarning) ? "planExpiresOn" : "planValidUntil"
        return "\(r.string(key)) \(date)"
    }

    private func colorLabel(_ s: AccountSummary) -> String {
        if !s.isActive { return "error" }
        if s.isExpiringSoon { return "orange" }
        if s.isExpiringWarning { return "warning" }
        return "accent"
    }

    /// GitHub renders `$\textcolor{…}{…}$` as colored text — the only color
    /// mechanism GFM tables / step summaries support (inline HTML styles are
    /// stripped). Source is ASCII, so char-count padding keeps the raw file
    /// monospace-aligned while the rendered table shows red/green.
    private func colored(_ text: String, _ color: String) -> String {
        "$\\textcolor{\(color)}{\\textsf{\(text)}}$"
    }

    private func yesNo(_ value: Bool) -> String {
        value ? colored("yes", "green") : colored("no", "red")
    }

    /// Renders the status-color token in its own colour.
    private func colorCell(_ label: String) -> String {
        let map = ["accent": "green", "warning": "orange", "orange": "orange", "error": "red"]
        return colored(label, map[label] ?? "gray")
    }

    @Test func textByScenario() throws {
        let r = try #require(Self.resolver, "xcstrings resolver unavailable")
        for sc in Self.scenarios {
            let s = AccountSummary.makeFake(daysRemaining: sc.daysRemaining, kind: sc.kind, isAutoRenew: false, baseAddress: "fake")
            let branch = expectedBranchKey(s)
            let text = displayedText(s, r)
            let daysCell = sc.daysRemaining.map(String.init) ?? "expired"

            ReportBuilder.shared.addTextRow([
                sc.planLabel,
                daysCell,
                yesNo(s.isActive),
                s.isActive ? yesNo(s.isExpiringSoon) : colored("n/a", "gray"),
                s.isActive ? yesNo(s.isExpiringWarning) : colored("n/a", "gray"),
                "`\(branch)`",
                text,
                colorCell(colorLabel(s))
            ])

            let produced = String((s.planValidUntilAttributedString.map { String($0.characters) }) ?? "")
            if branch == "noActivePlan" {
                #expect(produced == "noActivePlan", "\(sc.planLabel) \(daysCell)")
            } else {
                #expect(produced.hasPrefix(branch),
                        "\(sc.planLabel) \(daysCell): expected branch \(branch), got \(produced)")
            }
        }
        ReportBuilder.shared.write()
    }

    @Test func buttonsByScenario() {
        // macOS scope (per spec). Vary auto-renew + linked to exercise visibility.
        let variants: [(autoRenew: Bool, linked: Bool)] = [(true, true), (false, false)]
        for sc in Self.scenarios {
            for v in variants {
                var s = AccountSummary.makeFake(daysRemaining: sc.daysRemaining, kind: sc.kind, isAutoRenew: v.autoRenew, baseAddress: "fake")
                s.isLinked = v.linked
                let buttons = accountButtons(for: s, platform: .macOS, isTestFlight: false)
                let daysCell = sc.daysRemaining.map(String.init) ?? "expired"
                let scenarioLabel = "\(sc.planLabel) · \(daysCell) · autoRenew=\(v.autoRenew) · linked=\(v.linked)"

                ReportBuilder.shared.addButtonRow([
                    scenarioLabel,
                    buttons.map { $0.kind.reportDescription }.joined(separator: "; ")
                ])

                #expect(buttons.last?.kind == .logout, "\(scenarioLabel)")
                #expect(buttons.last?.isDestructive == true, "\(scenarioLabel)")
                #expect(buttons.contains(where: { $0.kind == .manageSubscriptionExternal }), "\(scenarioLabel)")
                #expect(buttons.contains(where: { $0.kind == .renewPlan }) == s.shouldShowRenewRow, "\(scenarioLabel)")
            }
        }
        ReportBuilder.shared.write()
    }

    /// Guards against "button set drift": the title keys wired into the view must
    /// equal the title keys the descriptor can emit (iOS superset, non-TestFlight).
    @Test func descriptorMatchesViewButtonSet() throws {
        var s = AccountSummary.makeFake(daysRemaining: nil, kind: .oneYear, isAutoRenew: false, baseAddress: "a")
        s.isLinked = false
        let descriptorKeys = Set(accountButtons(for: s, platform: .iOS, isTestFlight: false).map(\.titleKey))

        let viewKeys = try Self.viewButtonTitleKeys()
        #expect(viewKeys == descriptorKeys,
                "Account view button title-keys diverged from the descriptor. View=\(viewKeys.sorted()) Descriptor=\(descriptorKeys.sorted())")
    }

    /// Extracts button title keys from the Account view sources:
    ///  - `title: "KEY".localizedString` on SettingsListItem button rows
    ///  - the renew row `Text("settings.account.renewNow".localizedString)`
    ///
    /// Non-button `title:` args (navbar `CustomNavBar`, copyable `SettingsCopyableContentCell`
    /// via the `cell(title:…)` helper) are excluded so the extracted set reflects only
    /// the tappable account buttons the descriptor mirrors.
    static func viewButtonTitleKeys(file: StaticString = #filePath) throws -> Set<String> {
        // file = …/nym-vpn-apple/ServicesMutual/Tests/ConnectionTypesTests/AccountReportTests.swift
        var root = URL(fileURLWithPath: "\(file)")
        for _ in 0..<4 { root.deleteLastPathComponent() } // → nym-vpn-apple/
        let dir = root.appendingPathComponent("Settings/Sources/Settings/AccountAndDevices")
        let files = [
            dir.appendingPathComponent("AccountAndDevicesView.swift"),
            dir.appendingPathComponent("AccountAndDevicesView+AccountStatus.swift")
        ]
        var source = ""
        for f in files { source += (try? String(contentsOf: f, encoding: .utf8)) ?? "" }

        // Non-button `title:` args we must NOT treat as buttons:
        //  - navbar title (CustomNavBar): "settings.account"
        //  - copyable content cells (cell(title:…) → SettingsCopyableContentCell):
        //    "settings.accountID", "settings.deviceId"
        //  - refresh-failure snackbar (SnackbarItem(title:…)): "settings.account.refreshFailed"
        //  - allowance-reached snackbar (SnackbarItem(title:…)): "settings.account.allowanceReached.title"
        let nonButtonTitleKeys: Set<String> = [
            "settings.account",
            "settings.accountID",
            "settings.deviceId",
            "settings.account.refreshFailed",
            "settings.account.allowanceReached.title"
        ]

        var keys = Set<String>()
        let titleRegex = try NSRegularExpression(pattern: #"title:\s*"([^"]+)"\.localizedString"#)
        for m in titleRegex.matches(in: source, range: NSRange(source.startIndex..., in: source)) {
            if let r = Range(m.range(at: 1), in: source) {
                let key = String(source[r])
                if !nonButtonTitleKeys.contains(key) { keys.insert(key) }
            }
        }
        if source.contains(#""settings.account.renewNow".localizedString"#) {
            keys.insert("settings.account.renewNow")
        }
        return keys
    }
}
