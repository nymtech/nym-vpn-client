#if SANTA
#if os(iOS)
import ErrorHandler
import NymVPNLib
import PathManager

extension CredentialsManager {
    /// Santa QA: POST `/device` with the identity already on disk. Does not log out
    /// or mint new keys.
    public func reregisterCurrentDevice() async throws -> String {
        guard isValidCredentialImported else {
            throw CredentialsManagerError.generalError("No account stored")
        }
        guard !isTunnelActive else {
            throw VPNErrorReason.accountStoreBusy
        }

        await shutdownControllersAndWait()

        let env = try resolvedNetworkEnvironment()
        let deviceId = try await Task {
            let dataDir = try PathManager.dataFolderURL().path()
            return try await NymVpnAccountStorage(
                dataDir: dataDir,
                environment: env
            ).registerDevice()
        }.value

        logger.info("Santa reregisterCurrentDevice deviceId=\(deviceId)")
        deviceIdentifier = deviceId
        await updateAccountSummary(force: true)
        return deviceId
    }
}
#endif
#endif
