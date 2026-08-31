import NymVPNLib

extension GRPCManager {
    public func socks5Status() async throws -> Socks5Status? {
        try await rpcClient?.getSocks5Status()
    }

    public func disableSocks5() async throws {
        try await rpcClient?.disableSocks5()
    }

    public func enableSocks5(
        socks5Settings: Socks5Settings,
        httpRpcSettings: HttpRpcSettings,
        exitPoint: ExitPoint
    ) async throws {
        try await rpcClient?.enableSocks5(
            socks5Settings: socks5Settings,
            httpRpcSettings: httpRpcSettings,
            exitPoint: exitPoint
        )
    }
}
