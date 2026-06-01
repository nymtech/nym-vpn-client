#if SANTA
import Combine
import SwiftUI
import AppSettings
import AppVersionProvider
import ConfigurationManager
import ConnectionTypes
import CredentialsManager
import NymLogger
#if os(iOS)
import NymVPNLib
#elseif os(macOS)
import GRPCManager
#endif
import Theme

@MainActor public final class SantasViewModel: ObservableObject {
    private let appSettings: AppSettings
    private let configurationManager: ConfigurationManager
#if os(macOS)
    private let grpcManager: GRPCManager
#endif

    private var cancellables = Set<AnyCancellable>()
    @Binding private var path: NavigationPath

    let title = "🎅 Santa's menu 🎅"

    /// QA: when true, applied fake summaries are marked auto-renewing
    /// (hides the renew CTA, shows the "auto-renews" note).
    @Published var fakeAutoRenew = false

    var actualEnv: String {
#if os(iOS)
        configurationManager.networkEnv?.current().nymNetwork.networkName ?? "unknown"
#elseif os(macOS)
        grpcManager.networkName ?? "Restart app to see"
#endif
    }

    var currentAppEnv: String {
        appSettings.currentEnv
    }

    var envs: [Env] {
        Env.allCases
    }

    var libVersion: String {
#if os(iOS)
        AppVersionProvider.libVersion
#elseif os(macOS)
        grpcManager.daemonVersion
#endif
    }

#if os(iOS)
    init(
        path: Binding<NavigationPath>,
        appSettings: AppSettings,
        configurationManager: ConfigurationManager
    ) {
        _path = path
        self.appSettings = appSettings
        self.configurationManager = configurationManager
    }
#elseif os(macOS)
    init(
        path: Binding<NavigationPath>,
        appSettings: AppSettings,
        configurationManager: ConfigurationManager,
        grpcManager: GRPCManager
    ) {
        _path = path
        self.appSettings = appSettings
        self.grpcManager = grpcManager
        self.configurationManager = configurationManager
    }
#endif

    func changeEnvironment(to env: Env) {
        configurationManager.updateEnv(to: env)
        objectWillChange.send()
    }

    // MARK: - Account-summary fakes (QA)

    /// A canned subscription "time-left" scenario. `daysRemaining == nil` ⇒ expired.
    struct AccountSummaryPreset {
        let label: String
        let daysRemaining: Int?
        let kind: VpnSubscriptionKind
    }

    /// Yearly plan (long-plan thresholds: warning < 60d, soon < 15d).
    let yearlyPresets: [AccountSummaryPreset] = [
        .init(label: "6 months", daysRemaining: 182, kind: .oneYear),
        .init(label: "2 months", daysRemaining: 61, kind: .oneYear),
        .init(label: "1 month", daysRemaining: 30, kind: .oneYear),
        .init(label: "2 weeks", daysRemaining: 14, kind: .oneYear),
        .init(label: "1 week", daysRemaining: 7, kind: .oneYear),
        .init(label: "3 days", daysRemaining: 3, kind: .oneYear),
        .init(label: "1 day", daysRemaining: 1, kind: .oneYear),
        .init(label: "Expired", daysRemaining: nil, kind: .oneYear)
    ]

    /// Monthly plan (short-plan thresholds: warning < 7d, soon < 2d).
    let monthlyPresets: [AccountSummaryPreset] = [
        .init(label: "1 month", daysRemaining: 30, kind: .oneMonth),
        .init(label: "2 weeks", daysRemaining: 14, kind: .oneMonth),
        .init(label: "6 days", daysRemaining: 6, kind: .oneMonth),
        .init(label: "3 days", daysRemaining: 3, kind: .oneMonth),
        .init(label: "1 day", daysRemaining: 1, kind: .oneMonth),
        .init(label: "Expired", daysRemaining: nil, kind: .oneMonth)
    ]

    func applyAccountSummaryPreset(_ preset: AccountSummaryPreset) {
        let baseAddress = CredentialsManager.shared.accountSummary?.accountAddress ?? "fake-account"
        let summary = Self.makeFakeSummary(
            daysRemaining: preset.daysRemaining,
            kind: preset.kind,
            isAutoRenew: fakeAutoRenew,
            baseAddress: baseAddress
        )
        CredentialsManager.shared.applyDebugAccountSummary(summary)
    }

    func clearAccountSummaryOverride() {
        CredentialsManager.shared.clearDebugAccountSummary()
    }

    static func makeFakeSummary(
        daysRemaining: Int?,
        kind: VpnSubscriptionKind,
        isAutoRenew: Bool,
        baseAddress: String
    ) -> AccountSummary {
        let now = Date()
        let isActive = (daysRemaining ?? -1) >= 0
        let validUntil = daysRemaining.map { now.addingTimeInterval(TimeInterval($0) * 86_400) }
        let subscription = Subscription(
            status: .active, // never .pending — pending hides the expiry banner
            subscription: VpnSubscription(
                createdOnUtc: now,
                lastUpdatedUtc: now,
                id: "fake-subscription",
                validUntilDate: validUntil ?? now,
                validFromDate: now,
                status: "active",
                kind: kind,
                isRecurring: isAutoRenew
            )
        )
        return AccountSummary(
            validUntilDate: validUntil,
            trafficUsedGb: nil,
            trafficLimitGb: nil,
            trafficResetDate: nil,
            accountAddress: baseAddress,
            cannonicalAccountAddress: nil,
            accountAuthMethod: [],
            isLinked: true,
            isActive: isActive,
            isAutoRenewEnabled: isAutoRenew,
            subscription: subscription
        )
    }

    func navigateBack() {
        if !path.isEmpty { path.removeLast() }
    }

    var logFilesSize: String {
        let fm = FileManager.default
        var totalSize: UInt64 = 0
        for type in LogFileType.allCases {
            guard let url = LogFileManager.logFileURL(logFileType: type),
                  let attrs = try? fm.attributesOfItem(atPath: url.path),
                  let size = attrs[.size] as? UInt64
            else { continue }
            totalSize += size
        }
        return ByteCountFormatter.string(fromByteCount: Int64(totalSize), countStyle: .file)
    }

#if os(macOS)
    func updateDaemonInfo() {
        Task {
            try? await grpcManager.version()
            Task { @MainActor in
                objectWillChange.send()
            }
        }
    }
#endif
}
#endif
