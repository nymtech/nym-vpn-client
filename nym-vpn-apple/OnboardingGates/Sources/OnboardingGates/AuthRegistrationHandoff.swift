import Foundation

public enum AuthRegistrationHandoffAction: Equatable, Sendable {
    case noop
    case startProcessing
    case promoteToInitialDrawer
}

public enum AuthRegistrationHandoff {
    public static func resolve(
        allowsCredentialPromotion: Bool,
        canStartProcessing: Bool,
        hasProcessingViewModel: Bool,
        isCredentialImported: Bool,
        processingComplete: Bool
    ) -> AuthRegistrationHandoffAction {
        guard !hasProcessingViewModel else { return .noop }
        if allowsCredentialPromotion, canStartProcessing {
            return .startProcessing
        }
        guard isCredentialImported else { return .noop }
        if canStartProcessing {
            return .startProcessing
        }
        if processingComplete {
            return .promoteToInitialDrawer
        }
        return .noop
    }
}
