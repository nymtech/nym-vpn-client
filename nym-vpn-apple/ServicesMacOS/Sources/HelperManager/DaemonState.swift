public enum DaemonState {
    case requiresManualRemoval
    case unknown
    case registered
    case requiresAuthorization
    case authorized
    case running
    case requiresUpdate
    case updating
}
