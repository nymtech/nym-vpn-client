import Combine
import Foundation
import CountriesManagerTypes
import Logging
#if os(iOS)
import MixnetLibrary
#elseif os(macOS)
import GRPCManager
#endif

public final class GatewayManager {
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

#if os(iOS)
    public init() {
        self.entry = []
        self.exit = []
        self.vpn = []

        setup()
    }
#elseif os(macOS)
    public init(grpcManager: GRPCManager = .shared) {
        self.grpcManager = grpcManager
        self.entry = []
        self.exit = []
        self.vpn = []

        setup()
    }
#endif
}

private extension GatewayManager {
    func setup() {
        updateGateways()
        func setupAutoUpdates() {
            timer = Timer.scheduledTimer(
                timeInterval: 600,
                target: self,
                selector: #selector(updateGateways),
                userInfo: nil,
                repeats: true
            )
        }
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

        Task(priority: .background) { [weak self] in
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
}
