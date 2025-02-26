import Foundation
import ErrorReason
import UIComponents

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

    @MainActor func navigateToAddCredetialsIfNeeded(error: Error?) {
        guard let errorReason = error as? ErrorReason,
              errorReason == .noAccountStored
        else {
            return
        }
        resetStatusInfoState()
        navigateToAddCredentials()
    }

    @MainActor func updateLastError(_ error: Error?) {
        if lastError == nil, let error {
            lastError = error
            updateStatusInfoState(with: .error(message: error.localizedDescription))
            navigateToAddCredetialsIfNeeded(error: error)
        } else {
            guard let lastNsError = lastError as? NSError,
                  let error = error as? NSError,
                  lastNsError.domain != error.domain,
                  lastNsError.code != error.code
            else {
                return
            }
            lastError = error
            updateStatusInfoState(with: .error(message: error.localizedDescription))
            navigateToAddCredetialsIfNeeded(error: error)
        }
    }
}
