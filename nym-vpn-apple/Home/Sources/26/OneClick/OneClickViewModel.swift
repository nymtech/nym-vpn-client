import Combine
import Foundation
import SwiftUI
import SnackbarManager
import AccountPrefetchGates
import AppSettings
import ConnectionManager
import ConnectionTypes
import CredentialsManager
import ErrorReason
import GatewayManager
import ImpactGenerator
import NetworkMonitor
import TunnelStatus
import UIComponents
#if os(macOS)
import GRPCManager
#endif

@MainActor
@Observable
public final class OneClickViewModel {
    private let appSettings: AppSettings
    private let connectionManager: ConnectionManager
    private let credentialsManager: CredentialsManager
    private let gatewayManager: GatewayManager
    private let snackbarManager: SnackbarManager
    private let impactGenerator: ImpactGenerator
    private let networkMonitor: NetworkMonitor
#if os(macOS)
    private let grpcManager: GRPCManager
#endif

    var connectState: OneClickConnectState = .disconnected
    var entrySelectionPhase: OneClickSelectionPhase = .selecting
    var selectionPhase: OneClickSelectionPhase = .selecting
    var isLiveConnection: Bool = false

    var displayMode: OneClickDisplayMode

    var speedMode: OneClickSpeedMode

    var showsIncompleteSubscriptionBanner: Bool {
        IAPFeedbackPolicy.shouldShowIncompleteSubscriptionBanner(
            isCredentialImported: credentialsManager.isValidCredentialImported,
            isAccountActive: credentialsManager.isAccountActive()
        )
    }

    /// Invoked when the daemon reports `.inactiveSubscription` or when the
    /// pre-flight gate detects an expired account. Routes the user into the
    /// purchase flow.
    @ObservationIgnored public var onRequestPlanPurchase: (() -> Void)?
    /// macOS only: invoked when a connect attempt is made while the helper
    /// daemon is not running, so the user can install/enable it.
    @ObservationIgnored public var onRequestDaemonEnable: (() -> Void)?

    @ObservationIgnored private var connectDisconnectTask: Task<Void, Never>?
    @ObservationIgnored private var resolveTask: Task<Void, Never>?
    @ObservationIgnored private var cancellables = Set<AnyCancellable>()
    @ObservationIgnored private var isConnectDisconnectInFlight = false

#if os(iOS)
    public init(
        appSettings: AppSettings,
        connectionManager: ConnectionManager,
        credentialsManager: CredentialsManager,
        gatewayManager: GatewayManager,
        snackbarManager: SnackbarManager,
        impactGenerator: ImpactGenerator,
        networkMonitor: NetworkMonitor
    ) {
        self.appSettings = appSettings
        self.connectionManager = connectionManager
        self.credentialsManager = credentialsManager
        self.gatewayManager = gatewayManager
        self.snackbarManager = snackbarManager
        self.impactGenerator = impactGenerator
        self.networkMonitor = networkMonitor

        self.displayMode = OneClickDisplayMode(rawValue: appSettings.oneClickDisplayModeRaw) ?? .powerUser
        self.speedMode = OneClickSpeedMode(
            isTwoHop: connectionManager.connectionConfig.enableTwoHop
        )

        seedFromCurrentValues()
        observe()
    }
#elseif os(macOS)
    public init(
        appSettings: AppSettings,
        connectionManager: ConnectionManager,
        credentialsManager: CredentialsManager,
        gatewayManager: GatewayManager,
        snackbarManager: SnackbarManager,
        impactGenerator: ImpactGenerator,
        networkMonitor: NetworkMonitor,
        grpcManager: GRPCManager
    ) {
        self.appSettings = appSettings
        self.connectionManager = connectionManager
        self.credentialsManager = credentialsManager
        self.gatewayManager = gatewayManager
        self.snackbarManager = snackbarManager
        self.impactGenerator = impactGenerator
        self.networkMonitor = networkMonitor
        self.grpcManager = grpcManager

        self.displayMode = OneClickDisplayMode(rawValue: appSettings.oneClickDisplayModeRaw) ?? .powerUser
        self.speedMode = OneClickSpeedMode(
            isTwoHop: connectionManager.connectionConfig.enableTwoHop
        )

        seedFromCurrentValues()
        observe()
    }
#endif

    func connectButtonTapped() {
        guard !isConnectDisconnectInFlight else { return }
        guard connectionManager.currentTunnelStatus != .disconnecting else { return }

        impactGenerator.impact()
        snackbarManager.clear()

        let isConnectingTap = connectionManager.currentTunnelStatus != .connected

        connectDisconnectTask?.cancel()
        connectDisconnectTask = Task { @MainActor [weak self] in
            guard let self else { return }
            isConnectDisconnectInFlight = true
            defer { isConnectDisconnectInFlight = false }

            if isConnectingTap {
#if os(iOS)
                if !networkMonitor.isAvailable {
                    presentOfflineAlert()
                    return
                }
#endif
#if os(macOS)
                if !grpcManager.isServing {
                    onRequestDaemonEnable?()
                    return
                }
#endif
                guard credentialsManager.isValidCredentialImported else { return }
                if await !credentialsManager.isAccountValid() {
                    await credentialsManager.updateAccountSummary()
                    let summary = credentialsManager.accountSummary
                    let shouldOfferPurchase = ConnectPlanPurchaseGatePolicy.shouldOfferPlanPurchaseOnConnect(
                        isAccountRegistrationInFlight: credentialsManager.isAccountRegistrationInFlight,
                        accountSummaryLastFetchFailed: credentialsManager.accountSummaryLastFetchFailed,
                        isAccountActive: credentialsManager.isAccountActive(),
                        validUntilIsFuture: LoginSessionPolicy.validUntilIsFuture(
                            validUntil: summary?.validUntilDate
                        ),
                        hasAccountSummary: summary != nil
                    )
                    if shouldOfferPurchase {
                        onRequestPlanPurchase?()
                        return
                    }
                }
            }

            do {
                try await connectionManager.connectDisconnect()
            } catch {
                impactGenerator.error()
                presentConnectionErrorAlert(
                    message: ConnectionStatusViewModel.userFacingMessage(from: error)
                )
            }
            handleInactiveSubscriptionErrorIfNeeded()
            clearLastErrorIfNeeded()
        }
    }

    func requestIndependenceConsent() {
        snackbarManager.enqueue(
            SnackbarItem(
                style: .warning,
                title: "gatewayIndependence.warning.title".localizedString,
                message: "gatewayIndependence.warning.message".localizedString,
                actionTitle: "gatewayIndependence.warning.connectAnyway".localizedString,
                onAction: { [weak self] in self?.independenceConsentAgreed() },
                secondaryActionTitle: "cancel".localizedString,
                duration: 15
            )
        )
    }

    func independenceConsentAgreed() {
        Task { @MainActor [weak self] in
            guard let self else { return }
            do {
                try await connectionManager.acceptRelaxedGatewayIndependence()
            } catch {
                impactGenerator.error()
                presentConnectionErrorAlert(
                    message: ConnectionStatusViewModel.userFacingMessage(from: error)
                )
            }
        }
    }

    func upCaretTapped() {
        guard displayMode == .powerUser else { return }
        impactGenerator.softImpact()
        applyDisplayMode(.nerd)
    }

    func downCaretTapped() {
        guard displayMode == .nerd else { return }
        impactGenerator.softImpact()
        applyDisplayMode(.powerUser)
    }

    func setSpeedMode(_ mode: OneClickSpeedMode) {
        guard mode != speedMode else { return }
        impactGenerator.softImpact()
        speedMode = mode

        let cfg = connectionManager.connectionConfig
        switch mode {
        case .fast:
            if !cfg.enableTwoHop {
                connectionManager.setTwoHop(true)
            }
        case .anonymous:
            if cfg.enableTwoHop {
                connectionManager.setTwoHop(false)
            }
        }
    }

    func incompleteSubscriptionBannerTapped() {
        impactGenerator.softImpact()
        onRequestPlanPurchase?()
    }
}

private extension OneClickViewModel {
    func seedFromCurrentValues() {
        recomputeConnectState()
        refreshSelection()
    }

    func observe() {
        connectionManager.$currentTunnelStatus
            .removeDuplicates()
            .receive(on: DispatchQueue.main)
            .sink { [weak self] status in
                guard let self else { return }
                if case .connected = status {
                    impactGenerator.success()
                }
                recomputeConnectState()
                refreshSelection()
            }
            .store(in: &cancellables)

        connectionManager.$connectionInfoData
            .receive(on: DispatchQueue.main)
            .sink { [weak self] _ in
                self?.refreshSelection()
            }
            .store(in: &cancellables)

        connectionManager.$connectionConfig
            .receive(on: DispatchQueue.main)
            .sink { [weak self] config in
                guard let self else { return }
                speedMode = OneClickSpeedMode(isTwoHop: config.enableTwoHop)
                refreshSelection()
            }
            .store(in: &cancellables)

#if SANTA
        Publishers.MergeMany(
            gatewayManager.$entry.map { _ in () },
            gatewayManager.$exit.map { _ in () },
            gatewayManager.$vpn.map { _ in () }
        )
        .receive(on: DispatchQueue.main)
        .sink { [weak self] in
            self?.refreshSelection()
        }
        .store(in: &cancellables)
#endif

        credentialsManager.$accountSummary
            .receive(on: DispatchQueue.main)
            .sink { [weak self] _ in
                self?.recomputeConnectState()
            }
            .store(in: &cancellables)

#if os(iOS)
        networkMonitor.$isAvailable
            .receive(on: DispatchQueue.main)
            .sink { [weak self] _ in
                self?.recomputeConnectState()
            }
            .store(in: &cancellables)
#endif

#if os(macOS)
        grpcManager.$isServing
            .receive(on: DispatchQueue.main)
            .sink { [weak self] _ in
                self?.recomputeConnectState()
            }
            .store(in: &cancellables)
#endif
    }

    func recomputeConnectState() {
        let next = derivedConnectState()
        guard next != connectState else { return }
        connectState = next
    }

    func derivedConnectState() -> OneClickConnectState {
        switch connectionManager.currentTunnelStatus {
        case .connected:
            return .connected
        case .disconnecting:
            return .disconnecting
        case .connecting, .reasserting, .restarting, .offlineReconnect, .error:
            return .stop
        case .disconnected, .offline, .unknown:
#if os(iOS)
            if !networkMonitor.isAvailable {
                return .noInternet
            }
#endif
            if credentialsManager.isValidCredentialImported, !credentialsManager.isAccountActive() {
                return .noSubscription
            }
            return .disconnected
        }
    }

    func handleInactiveSubscriptionErrorIfNeeded() {
        guard connectionManager.currentTunnelStatus == .error else { return }
        guard let error = connectionManager.lastError else { return }
        let reason: ErrorReason?
        if let typed = error as? ErrorReason {
            reason = typed
        } else {
            let nsError = error as NSError
            reason = nsError.domain == ErrorReason.domain ? ErrorReason(nsError: nsError) : nil
        }
        guard reason == .inactiveSubscription else { return }
        onRequestPlanPurchase?()
    }

    func clearLastErrorIfNeeded() {
        switch connectionManager.currentTunnelStatus {
        case .disconnecting, .disconnected, .error:
            connectionManager.lastError = nil
        default:
            break
        }
    }

    func applyDisplayMode(_ mode: OneClickDisplayMode) {
        displayMode = mode
        appSettings.oneClickDisplayModeRaw = mode.rawValue
        refreshSelection()
    }

    func refreshSelection() {
        let cfg = connectionManager.connectionConfig
        let twoHop = cfg.enableTwoHop
        let exitType: NodeType = twoHop ? .vpn : .exit
        let entryType: NodeType = twoHop ? .vpn : .entry
        let info = connectionManager.connectionInfoData
        let liveCandidate = isLiveStatus(connectionManager.currentTunnelStatus) && info != nil
        resolveTask?.cancel()
        resolveTask = Task { @MainActor [weak self] in
            guard let self else { return }
            if liveCandidate,
               let info,
               let exitLive = livePhase(
                   gatewayId: info.exitGatewayId,
                   gatewayType: exitType,
                   hopType: .exit,
                   displayMode: displayMode
               ) {
                isLiveConnection = true
                selectionPhase = exitLive
                if displayMode == .nerd {
                    entrySelectionPhase = livePhase(
                        gatewayId: info.entryGatewayId,
                        gatewayType: entryType,
                        hopType: .entry,
                        displayMode: displayMode
                    )
                        ?? resolveEntryPhase(
                            entry: cfg.entry,
                            gatewayId: info.entryGatewayId ?? cfg.entry.gatewayId,
                            gatewayType: entryType
                        )
                } else {
                    entrySelectionPhase = .selecting
                }
                return
            }
            isLiveConnection = false
            switch displayMode {
            case .powerUser:
                entrySelectionPhase = .selecting
                selectionPhase = resolveExitPhase(
                    exit: cfg.exit,
                    gatewayId: cfg.exit.gatewayId,
                    gatewayType: exitType
                )
            case .nerd:
                entrySelectionPhase = resolveEntryPhase(
                    entry: cfg.entry,
                    gatewayId: cfg.entry.gatewayId,
                    gatewayType: entryType
                )
                selectionPhase = resolveExitPhase(
                    exit: cfg.exit,
                    gatewayId: cfg.exit.gatewayId,
                    gatewayType: exitType
                )
            }
        }
    }

    func isLiveStatus(_ status: TunnelStatus) -> Bool {
        switch status {
        case .connecting, .reasserting, .restarting, .connected, .offlineReconnect, .disconnecting:
            return true
        case .disconnected, .offline, .error, .unknown:
            return false
        }
    }

    func livePhase(
        gatewayId: String?,
        gatewayType: NodeType,
        hopType: HopType,
        displayMode: OneClickDisplayMode
    ) -> OneClickSelectionPhase? {
        guard let gatewayId,
              let gateway = gatewayManager.gateway(with: gatewayId, gatewayType: gatewayType)
        else { return nil }
        let title: String
        switch displayMode {
        case .nerd:
            title = gateway.name ?? gateway.id
        case .powerUser:
            title = gateway.ipv4s.first ?? gateway.name ?? gateway.id
        }
        let location = gateway.location
        let subtitle: String?
        if let location {
            let country = gatewayManager.localizedCountry(with: location.twoLetterIsoCountryCode)?.name
            if gatewayManager.shouldDisplayRegion(with: location.twoLetterIsoCountryCode) {
                subtitle = "\(location.city), \(location.region), \(country ?? "")"
            } else {
                subtitle = "\(location.city), \(country ?? "")"
            }
        } else {
            subtitle = nil
        }
        return .selected(OneClickServerInfo(
            countryCode: location?.twoLetterIsoCountryCode ?? "",
            title: title,
            subtitle: subtitle,
            score: score(for: gateway),
            gateway: gateway,
            hopType: hopType,
            showsInfoButton: true
        ))
    }

    func resolveEntryPhase(entry: EntryGateway, gatewayId: String?, gatewayType: NodeType) -> OneClickSelectionPhase {
        let gateway = gatewayId.flatMap { gatewayManager.gateway(with: $0, gatewayType: gatewayType) }
        let title: String
        if case .random = entry {
            title = "gatewaysView.random".localizedString
        } else {
            title = gatewayManager.userFriendlyTitle(with: entry)
                ?? gateway?.location?.city ?? gateway?.name ?? gateway?.id ?? ""
        }
        let subtitle = entrySubtitle(entry: entry, gateway: gateway)
        let countryCode = gatewayManager.countryCode(with: entry)
            ?? gateway?.location?.twoLetterIsoCountryCode ?? ""
        let scoreGateway = scoreGateway(forEntry: entry, fallback: gateway, gatewayType: gatewayType)
        let isExplicitGateway: Bool
        if case .gateway = entry { isExplicitGateway = true } else { isExplicitGateway = false }
        return makePhase(
            title: title,
            subtitle: subtitle,
            countryCode: countryCode,
            scoreGateway: scoreGateway,
            hopType: .entry,
            showsInfoButton: isExplicitGateway
        )
    }

    func resolveExitPhase(exit: ExitRouter, gatewayId: String?, gatewayType: NodeType) -> OneClickSelectionPhase {
        let gateway = gatewayId.flatMap { gatewayManager.gateway(with: $0, gatewayType: gatewayType) }
        let title: String
        if case .random = exit {
            title = "gatewaysView.random".localizedString
        } else {
            title = gatewayManager.userFriendlyTitle(with: exit)
                ?? gateway?.location?.city ?? gateway?.name ?? gateway?.id ?? ""
        }
        let subtitle = exitSubtitle(exit: exit, gateway: gateway)
        let countryCode = gatewayManager.countryCode(with: exit)
            ?? gateway?.location?.twoLetterIsoCountryCode ?? ""
        let scoreGateway = scoreGateway(forExit: exit, fallback: gateway, gatewayType: gatewayType)
        let isExplicitGateway: Bool
        if case .gateway = exit { isExplicitGateway = true } else { isExplicitGateway = false }
        return makePhase(
            title: title,
            subtitle: subtitle,
            countryCode: countryCode,
            scoreGateway: scoreGateway,
            hopType: .exit,
            showsInfoButton: isExplicitGateway
        )
    }

    func scoreGateway(forEntry entry: EntryGateway, fallback: GatewayNode?, gatewayType: NodeType) -> GatewayNode? {
        switch entry {
        case .country, .region, .random:
            return gatewayManager.bestGateway(matching: entry, gatewayType: gatewayType) ?? fallback
        case .gateway:
            return fallback
        }
    }

    func scoreGateway(forExit exit: ExitRouter, fallback: GatewayNode?, gatewayType: NodeType) -> GatewayNode? {
        switch exit {
        case .country, .region, .random:
            return gatewayManager.bestGateway(matching: exit, gatewayType: gatewayType) ?? fallback
        case .gateway:
            return fallback
        }
    }

    func makePhase(
        title: String,
        subtitle: String?,
        countryCode: String,
        scoreGateway: GatewayNode?,
        hopType: HopType,
        showsInfoButton: Bool = false
    ) -> OneClickSelectionPhase {
        if title.isEmpty && (subtitle?.isEmpty ?? true) {
            return .selecting
        }
        return .selected(OneClickServerInfo(
            countryCode: countryCode,
            title: title,
            subtitle: subtitle,
            score: scoreGateway.map { score(for: $0) } ?? .offline,
            gateway: scoreGateway,
            hopType: hopType,
            showsInfoButton: showsInfoButton
        ))
    }

    func score(for gateway: GatewayNode) -> OneClickServerScore {
        let raw: GatewayNodeScore?
        switch connectionManager.connectionType {
        case .wireguard:
            raw = gateway.performance?.score
        case .mixnet5hop:
            raw = gateway.performance?.mixnetScore ?? gateway.mixnetScore
        }
        return Self.score(from: raw ?? .offline)
    }

    func entrySubtitle(entry: EntryGateway, gateway: GatewayNode?) -> String? {
        guard let location = gateway?.location else { return nameOrId(gateway: gateway) }
        switch entry {
        case let .country(code):
            return countrySubtitle(gateway: gateway, countryCode: code, location: location)
        case .region:
            return regionSubtitle(gateway: gateway, location: location)
        case .gateway:
            return serverSubtitle(location: location, countryCode: location.twoLetterIsoCountryCode)
        case .random:
            return nil
        }
    }

    func exitSubtitle(exit: ExitRouter, gateway: GatewayNode?) -> String? {
        guard let location = gateway?.location else { return nameOrId(gateway: gateway) }
        switch exit {
        case let .country(code):
            return countrySubtitle(gateway: gateway, countryCode: code, location: location)
        case .region:
            return regionSubtitle(gateway: gateway, location: location)
        case .gateway:
            return serverSubtitle(location: location, countryCode: location.twoLetterIsoCountryCode)
        case .random:
            return nil
        }
    }

    func countrySubtitle(gateway: GatewayNode?, countryCode: String, location: GatewayNodeLocation) -> String? {
        if gatewayManager.shouldDisplayRegion(with: countryCode) {
            "\(location.city), \(location.region) \(nameOrId(gateway: gateway))"
        } else {
            "\(location.city) \(nameOrId(gateway: gateway))"
        }
    }

    func regionSubtitle(gateway: GatewayNode?, location: GatewayNodeLocation) -> String? {
        "\(location.city) \(nameOrId(gateway: gateway))"
    }

    func serverSubtitle(location: GatewayNodeLocation, countryCode: String) -> String? {
        let country = gatewayManager.localizedCountry(with: countryCode)
        if gatewayManager.shouldDisplayRegion(with: countryCode) {
            return "\(location.city), \(location.region), \(country?.name ?? "")"
        } else {
            return "\(location.city), \(country?.name ?? "")"
        }
    }

    func nameOrId(gateway: GatewayNode?) -> String {
        if let name = gateway?.name {
            "(\(name))"
        } else if let identifier = gateway?.id {
            "(\(identifier))"
        } else {
            ""
        }
    }

    static func score(from node: GatewayNodeScore) -> OneClickServerScore {
        switch node {
        case .high:
            return .high
        case .medium:
            return .medium
        case .low:
            return .low
        case .offline, .noScore:
            return .offline
        }
    }

    func presentOfflineAlert() {
        snackbarManager.enqueue(
            SnackbarItem(
                style: .warning,
                title: "home.modal.noInternetConnection.title".localizedString,
                message: "home.modal.noInternetConnection.subtitle".localizedString
            )
        )
    }

    func presentConnectionErrorAlert(message: String) {
        snackbarManager.enqueue(
            SnackbarItem(
                style: .critical,
                title: "connectionError.title".localizedString,
                message: ConnectionErrorCopy.message(reason: message),
                actionTitle: "disconnect".localizedString,
                onAction: { [weak self] in self?.connectButtonTapped() },
                duration: 7
            )
        )
    }
}

private extension OneClickSpeedMode {
    init(isTwoHop: Bool) {
        self = isTwoHop ? .fast : .anonymous
    }
}
