#if SANTA
import Combine
import SwiftUI
import AppSettings
import AppVersionProvider
import ConfigurationManager
import ConnectionTypes
import CredentialsManager
import AccountPrefetchGates
import NymLogger
import SnackbarManager
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
    /// (hides the renew CTA, shows the "auto-renews" note). Flipping this while
    /// an override is active re-fakes the current preset immediately so the
    /// toggle isn't misleading.
    @Published var fakeAutoRenew = false {
        didSet { reapplyAccountSummaryOverrideIfNeeded() }
    }

    @Published var isReregisteringDevice = false

    /// The preset currently faked, so a `fakeAutoRenew` flip can re-apply it.
    private var appliedPreset: AccountSummaryPreset?

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

#if os(iOS)
    var storeKitAccountGuidance: String {
        SantaStoreKitEnvironmentPolicy.guidanceMessage(
            isTestFlight: configurationManager.isTestFlight
        )
    }
#endif

    var libVersion: String {
#if os(iOS)
        AppVersionProvider.realAppVersion()
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
        let summary = AccountSummary.makeFake(
            daysRemaining: preset.daysRemaining,
            kind: preset.kind,
            isAutoRenew: fakeAutoRenew,
            baseAddress: baseAddress
        )
        appliedPreset = preset
        CredentialsManager.shared.applyDebugAccountSummary(summary)
    }

    func clearAccountSummaryOverride() {
        appliedPreset = nil
        CredentialsManager.shared.clearDebugAccountSummary()
    }

    /// Re-fake the active preset (e.g. after `fakeAutoRenew` toggles). No-op
    /// unless a preset is faked and the override is still live.
    private func reapplyAccountSummaryOverrideIfNeeded() {
        guard let appliedPreset,
              CredentialsManager.shared.isAccountSummaryOverridden
        else { return }
        applyAccountSummaryPreset(appliedPreset)
    }

    func navigateBack() {
        if !path.isEmpty { path.removeLast() }
    }

    func reregisterCurrentDevice() {
        guard !isReregisteringDevice else { return }
        isReregisteringDevice = true
        Task {
            defer { isReregisteringDevice = false }
#if os(iOS)
            do {
                let deviceId = try await CredentialsManager.shared.reregisterCurrentDevice()
                SnackbarManager.shared.enqueue(
                    SnackbarItem(
                        style: .confirmation,
                        title: "Device re-registered",
                        message: deviceId
                    )
                )
            } catch {
                SnackbarManager.shared.enqueue(
                    SnackbarItem(
                        style: .negative,
                        title: "Device re-register failed",
                        message: error.localizedDescription
                    )
                )
            }
#else
            SnackbarManager.shared.enqueue(
                SnackbarItem(
                    style: .warning,
                    title: "Device re-register is iOS only"
                )
            )
#endif
        }
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
}
#endif
