import Combine
import Foundation
import Network
import NetworkExtension
import ConnectionManager
import Tunnels
import TunnelStatus

public final class NetworkMonitor: ObservableObject {
    private let connectionManager: ConnectionManager
    private let monitor = NWPathMonitor()
    private let monitorQueue = DispatchQueue(label: String(describing: NetworkMonitor.self))

    @Published public private(set) var isAvailable = true
    @Published public private(set) var connectionType: NWInterface.InterfaceType?
    private var cancellables = Set<AnyCancellable>()

    public static let shared = NetworkMonitor()

    init(connectionManager: ConnectionManager = .shared) {
        self.connectionManager = connectionManager
        setup()
    }
}

// MARK: - Setup -
private extension NetworkMonitor {
    func setup() {
        setupNetworkMonitor()
        setupCurrentTunnelStatusObserver()

        monitor.start(queue: monitorQueue)
    }

    func setupNetworkMonitor() {
        monitor.pathUpdateHandler = { [weak self] path in
            let isConnected = path.status == .satisfied || path.status == .requiresConnection
            let interfaceType = NWInterface.InterfaceType.allCases.first { path.usesInterfaceType($0) }

            guard self?.connectionManager.currentTunnelStatus != .connected,
                isConnected != self?.isAvailable || interfaceType != self?.connectionType
            else {
                return
            }
            self?.isAvailable = isConnected
            self?.connectionType = interfaceType
        }
    }

    func setupCurrentTunnelStatusObserver() {
        connectionManager.$currentTunnelStatus
            .removeDuplicates()
            .sink { [weak self] status in
                guard let self else { return }

                switch status {
                case .connected, .connecting, .reasserting, .restarting, .unknown:
                    isAvailable = true
                case .offline, .offlineReconnect:
                    isAvailable = false
                case .disconnected, .disconnecting:
                    break
                }
            }
        .store(in: &cancellables)
    }
}

extension NWInterface.InterfaceType: @retroactive CaseIterable {
    public static var allCases: [NWInterface.InterfaceType] = [
        .other,
        .wifi,
        .cellular,
        .loopback,
        .wiredEthernet
    ]
}
