import UIComponents
#if os(iOS)
import ErrorHandler
#endif

extension HomeViewModel {
    func resetStatusInfoState() {
        updateStatusInfoState(with: .unknown)
    }

    func updateStatusInfoState(with newState: StatusInfoState) {
        Task { @MainActor in
            guard newState != statusInfoState else { return }
            statusInfoState = newState
        }
    }

    func navigateToAddCredetialsIfNeeded(error: Error?) {
#if os(iOS)
        if let vpnError = error as? VPNErrorReason, vpnError == .noAccountStored {
            Task { @MainActor [weak self] in
                self?.navigateToAddCredentials()
            }
        }
#endif
    }
}
