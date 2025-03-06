import Combine
import Foundation
import AppSettings
import ConfigurationManager
import CountriesManagerTypes
import Logging
#if os(iOS)
import MixnetLibrary
#elseif os(macOS)
import GRPCManager
#endif

public final class GatewayManager {
    let appSettings: AppSettings
    let configurationManager: ConfigurationManager
#if os(macOS)
    let grpcManager: GRPCManager
#endif
    let logger = Logger(label: "GatewayManager")

    var isLoading = false
    var timer: Timer?
    var gatewayStore = GatewayNodeStore()
    var cancellables = Set<AnyCancellable>()

    public static let shared = GatewayManager()

    @Published public var entry: [GatewayNode]
    @Published public var exit: [GatewayNode]
    @Published public var vpn: [GatewayNode]
    @Published public var lastError: Error?

#if os(iOS)
    public init(appSettings: AppSettings = .shared, configurationManager: ConfigurationManager = .shared) {
        self.appSettings = appSettings
        self.configurationManager = configurationManager
        self.entry = []
        self.exit = []
        self.vpn = []
        loadGatewayStore()
        loadPrebundledServersIfNecessary()
    }
#elseif os(macOS)
    public init(
        appSettings: AppSettings = .shared,
        configurationManager: ConfigurationManager = .shared,
        grpcManager: GRPCManager = .shared
    ) {
        self.appSettings = appSettings
        self.configurationManager = configurationManager
        self.grpcManager = grpcManager
        self.entry = []
        self.exit = []
        self.vpn = []
        loadGatewayStore()
        loadPrebundledServersIfNecessary()
    }
#endif

    public func setup() {
        updateGateways()
        setupAutoUpdates()
        configureEnvironmentChange()
    }
}

private extension GatewayManager {
    func setupAutoUpdates() {
        timer = Timer.scheduledTimer(
            timeInterval: 600,
            target: self,
            selector: #selector(updateGateways),
            userInfo: nil,
            repeats: true
        )
    }

    @objc func updateGateways() {
        guard !isLoading, needsReload()
        else {
            if entry.isEmpty
                || exit.isEmpty
                || vpn.isEmpty {
                loadGatewaysFromStore()
            }
            return
        }
        isLoading = true

        Task { [weak self] in
            await self?.fetchGateways()
        }
    }
    func needsReload() -> Bool {
        guard let lastFetchDate = gatewayStore.lastFetchDate else { return true }
        return isLongerThan30Minutes(date: lastFetchDate)
    }

    func isLongerThan30Minutes(date: Date) -> Bool {
        Date().timeIntervalSince(date) > 1800 ? true : false
    }

    func loadGatewaysFromStore() {
        Task { @MainActor in
            exit = gatewayStore.exit
            entry = gatewayStore.entry
            vpn = gatewayStore.vpn
        }
    }

    func configureEnvironmentChange() {
        configurationManager.environmentDidChange = { [weak self] in
            self?.gatewayStore.lastFetchDate = nil
            Task {
                await self?.fetchGateways()
            }
        }
    }
}

extension GatewayManager {
    func updateError(with error: Error) {
        Task { @MainActor in
            lastError = error
        }
    }
}
