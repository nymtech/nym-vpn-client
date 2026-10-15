/// The Nym account's free trial verdict, held in memory only: it flips the moment the account buys anything.
public enum NymTrialEligibility: Equatable, Sendable {
    case unknown
    case eligible
    case notEligible
}

/// How a free trial label is drawn.
public enum FreeTrialLabelVisibility: Equatable, Sendable {
    /// Both StoreKit and Nym allow a trial.
    case shown
    /// StoreKit allows a trial and the Nym verdict is still loading: keep the space, draw nothing.
    case reserved
    case hidden

    public var isShown: Bool { self == .shown }
}

public enum FreeTrialLabelPolicy {
    /// A trial is labelled only when StoreKit and the Nym account both allow it, so the label never
    /// promises a trial the purchase sheet will not show.
    public static func visibility(
        isStoreKitEligible: Bool,
        nymEligibility: NymTrialEligibility
    ) -> FreeTrialLabelVisibility {
        guard isStoreKitEligible else { return .hidden }
        switch nymEligibility {
        case .eligible:
            return .shown
        case .unknown:
            return .reserved
        case .notEligible:
            return .hidden
        }
    }
}
