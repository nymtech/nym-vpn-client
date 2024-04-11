import Combine
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

    // TODO: need param to separate mixnet/2hop/5hop
    public func connectDisconnect() {
        if
            let activeTunnel = tunnelsManager.activeTunnel,
            activeTunnel.status == .connected || activeTunnel.status == .connecting {
            print("🔥 Disconnect")
            disconnect(tunnel: activeTunnel)
        } else {
            // TODO: separate mixnet/2hop/5hop
            print("🔥 Connect")
            connectMixnet()
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
    func addMixnetConfigurationAndConnect() {
        guard let mixnetConfiguration = mixnetConfiguration() else { return }
        tunnelsManager.add(tunnelConfiguration: mixnetConfiguration) { [weak self] result in
            switch result {
            case .success(let tunnel):
                print(tunnel)
                self?.currentTunnel = tunnel
                self?.tunnelsManager.connect(tunnel: tunnel)
            case .failure(let error):
                print("Error: \(error)")
            }
        }
    }
}

private extension ConnectionManager {
    func connectMixnet() {
        guard let mixnetConfiguration = mixnetConfiguration() else { return }

        if let tunnel = tunnelsManager.tunnels.first(where: { $0.name == mixnetConfiguration.name }) {
            currentTunnel = tunnel
            tunnelsManager.connect(tunnel: tunnel)
        } else {
            addMixnetConfigurationAndConnect()
        }
    }

    func connectWireguard() {}

    func disconnect(tunnel: Tunnel) {
        tunnelsManager.disconnect(tunnel: tunnel)
    }
}

// MARK: - Testing -
private extension ConnectionManager {
    func mixnetConfiguration() -> MixnetConfig? {
        MixnetConfig(
            apiUrl: "https://sandbox-nym-api1.nymtech.net/api",
            explorerURL: "https://sandbox-explorer.nymtech.net/api",
            entryGateway: .location("DE"),
            exitRouter: .location("DE")
        )
    }
}
