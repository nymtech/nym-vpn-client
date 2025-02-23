import UIComponents
import ErrorReason

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
        if let errorReason = error as? ErrorReason,
           errorReason == .noAccountStored {
            Task { @MainActor [weak self] in
                self?.navigateToAddCredentials()
            }
        }
    }
}
