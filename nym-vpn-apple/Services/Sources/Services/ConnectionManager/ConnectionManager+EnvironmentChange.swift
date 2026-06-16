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
        entryGateway = .random
        exitRouter = .random
        connectionConfig = connectionStorage.connectionConfig
    }
}
#endif
