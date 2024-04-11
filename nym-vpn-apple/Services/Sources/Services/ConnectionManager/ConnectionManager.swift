import Combine
import TunnelMixnet
import Tunnels
import TunnelStatus

public final class ConnectionManager: ObservableObject {
    private let tunnelsManager: TunnelsManager

    public static let shared = ConnectionManager(tunnelsManager: TunnelsManager.shared)

    private var cancellables = Set<AnyCancellable>()

    @Published public var isTunnelManagerLoaded: Result<Void, Error>?
    @Published public var currentTunnel: Tunnel? {
        didSet {
            guard let currentTunnel else { return }
            configureTunnelStatusObserver(tunnel: currentTunnel)
        }
    }
    @Published public var currentTunnelStatus: TunnelStatus?

    public init(tunnelsManager: TunnelsManager = TunnelsManager.shared) {
        self.tunnelsManager = tunnelsManager
        setup()
    }

    public func connectDisconnect(with config: MixnetConfig) {
        if
            let activeTunnel = currentTunnel,
            activeTunnel.status == .connected || activeTunnel.status == .connecting {
            disconnect(tunnel: activeTunnel)
        } else {
            connectMixnet(with: config)
        }
    }
}

// MARK: - Setup -
private extension ConnectionManager {
    func setup() {
        setupTunnelManagerObservers()
    }

    func setupTunnelManagerObservers() {
        tunnelsManager.$isLoaded.sink { [weak self] isLoaded in
            self?.isTunnelManagerLoaded = isLoaded
        }
        .store(in: &cancellables)

        tunnelsManager.$activeTunnel.sink { [weak self] tunnel in
            self?.currentTunnel = tunnel
        }
        .store(in: &cancellables)
    }

    func configureTunnelStatusObserver(tunnel: Tunnel) {
        tunnel.$status.sink { [weak self] status in
            self?.currentTunnelStatus = status
        }
        .store(in: &cancellables)
    }
}

// MARK: - Tunnel config -
private extension ConnectionManager {
    func addMixnetConfigurationAndConnect(with config: MixnetConfig) {
        tunnelsManager.add(tunnelConfiguration: config) { [weak self] result in
            switch result {
            case .success(let tunnel):
                self?.currentTunnel = tunnel
                self?.tunnelsManager.connect(tunnel: tunnel)
            case .failure(let error):
                // TODO: handle error
                print("Error: \(error)")
            }
        }
    }
}

private extension ConnectionManager {
    func connectMixnet(with config: MixnetConfig) {
        if let tunnel = tunnelsManager.tunnels.first(where: { $0.name == config.name }) {
            currentTunnel = tunnel
            tunnelsManager.connect(tunnel: tunnel)
        } else {
            addMixnetConfigurationAndConnect(with: config)
        }
    }

    func connectWireguard() {}

    func disconnect(tunnel: Tunnel) {
        tunnelsManager.disconnect(tunnel: tunnel)
    }
}
