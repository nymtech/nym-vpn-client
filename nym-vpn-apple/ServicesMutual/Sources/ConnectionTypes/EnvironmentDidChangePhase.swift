#if SANTA
import Foundation

public enum EnvironmentDidChangePhase: Int, Comparable, Sendable {
    case connectionConfig = 0
    case featureFlags = 1
    case gateways = 2

    public static func < (lhs: EnvironmentDidChangePhase, rhs: EnvironmentDidChangePhase) -> Bool {
        lhs.rawValue < rhs.rawValue
    }
}

public enum EnvironmentDidChangeDispatch {
    public static func invokeInOrder(
        _ observers: [(phase: EnvironmentDidChangePhase, action: () -> Void)]
    ) {
        observers.sorted { $0.phase < $1.phase }.forEach { $0.action() }
    }
}
#endif
