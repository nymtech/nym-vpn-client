#if SANTA
import ConnectionTypes

@MainActor
public extension ConnectionManager {
    func registerForEnvironmentChanges() {
        connectionStorage.registerForEnvironmentChanges { [weak self] in
            self?.resetGatewaySelectionsForEnvironmentChange()
        }
    }

    func resetGatewaySelectionsForEnvironmentChange() {
        connectionStorage.resetGatewaySelectionsForEnvironmentChange()
        entryGateway = .auto
        exitRouter = .auto
        connectionConfig = connectionStorage.connectionConfig
    }
}
#endif
