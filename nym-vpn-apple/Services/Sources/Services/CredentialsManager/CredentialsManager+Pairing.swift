#if os(iOS)
import Foundation
import ErrorHandler
import NymVPNLib
import PathManager
import AppVersionProvider

// A pairing session lives in one controller, so it stays open until the session ends.
extension CredentialsManager {
    var accountControllerEvents: AsyncStream<TunnelEvent> {
        accountControllerEventListener.stream
    }

    func startPairing() async throws -> QrUrl {
        try await openPairingController().startPairing()
    }

    func scannedPairingKey(_ publicKey: String) async throws -> QrPairingConfirmation {
        try await openPairingController().scannedKey(pubKey: publicKey)
    }

    func verifiedPairingPin(sessionId: UInt32) async throws {
        guard let pairingController else { throw VpnError.PairingNotStarted }
        try await pairingController.verifiedPairingPin(sessionId: sessionId)
    }

    func cancelPairingProcess() async throws {
        try await pairingController?.cancelPairingProcess()
    }

    func closePairingController() async {
        guard let controller = pairingController else { return }
        pairingController = nil
        await controller.shutdownAndWait()
    }

    private func openPairingController() async throws -> NymAccountController {
        if let pairingController {
            return pairingController
        }
        await accountControllerTask?.value
        if let pairingController {
            return pairingController
        }
        guard !isTunnelActive else {
            throw VPNErrorReason.accountStoreBusy
        }
        let env = try resolvedRegistrationEnvironment()
        let dataDir = try PathManager.dataFolderURL().path()
        let offlineMonitor = await NymOfflineMonitor()
        let controller = try await NymAccountController(
            dataDir: dataDir,
            userAgent: .appUserAgent,
            networkEnv: env,
            offlineMonitor: offlineMonitor,
            eventListener: accountControllerEventListener
        )
        pairingController = controller
        return controller
    }
}
#endif
