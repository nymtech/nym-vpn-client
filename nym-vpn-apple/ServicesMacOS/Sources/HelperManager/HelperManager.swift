import SecurityFoundation
import ServiceManagement
import Shell

// Any changes made to Info.plist & Launchd.plist - are used to create daemon in nym-vpnd.

public final class HelperManager {
    private let secondInNanoseconds: UInt64 = 1000000000
    public static let shared = HelperManager()
    public let requiredVersion = "1.0.0-alpha.1"

    private var helperName = ""

    public func setup(helperName: String) {
        self.helperName = helperName
    }

    public func isHelperAuthorizedAndRunning() -> Bool {
        isHelperAuthorized() && isHelperRunning()
    }

    public func installHelperIfNeeded() async throws -> Bool {
        if isHelperAuthorizedAndRunning() {
            return true
        } else {
            do {
                _ = try authorizeAndInstallHelper()

                var retryCount = 0
                while retryCount < 10 {
                    retryCount += 1
                    if isHelperAuthorizedAndRunning() {
                        // Hack: Wait for daemon to start, to avoid connect button unresponsivness
                        try? await Task.sleep(nanoseconds: secondInNanoseconds * 5)
                        return true
                    }
                    try? await Task.sleep(nanoseconds: secondInNanoseconds)
                }
                return false
            }
        }
    }

    public func uninstallHelper() -> Bool {
        let domain = kSMDomainSystemLaunchd
        var authRef: AuthorizationRef?
        let status = AuthorizationCreate(nil, nil, [], &authRef)

        guard status == errAuthorizationSuccess,
              let authorization = authRef
        else {
            return false
        }

        var cfError: Unmanaged<CFError>?
        return SMJobRemove(domain, helperName as CFString, authorization, true, &cfError)
    }
}

private extension HelperManager {
    func authorizeAndInstallHelper() throws -> Bool {
        var authRef: AuthorizationRef?
        let status = AuthorizationCreate(nil, nil, [], &authRef)
        guard status == errAuthorizationSuccess, let authRef = authRef
        else {
            return false
        }

        var cfError: Unmanaged<CFError>?

        let rightName = kSMRightBlessPrivilegedHelper

        let result = rightName.withCString { cStringName -> Bool in
            var authItem = AuthorizationItem(
                name: cStringName,
                valueLength: 0,
                value: nil,
                flags: 0
            )

            return withUnsafeMutablePointer(to: &authItem) { authItemPointer -> Bool in
                var authRights = AuthorizationRights(count: 1, items: authItemPointer)
                let authFlags: AuthorizationFlags = [.interactionAllowed, .preAuthorize, .extendRights]
                let status = AuthorizationCopyRights(authRef, &authRights, nil, authFlags, nil)
                if status == errAuthorizationSuccess {
                    // Place to execute your authorized action:
                    return installHelper(with: authRef, error: &cfError)
                }
                return false
            }
        }

        if let error = cfError?.takeRetainedValue() {
            throw error
        }
        return result
    }

    func installHelper(with authRef: AuthorizationRef?, error: inout Unmanaged<CFError>?) -> Bool {
        // TODO: refactor using SMAPPService
        if !SMJobBless(kSMDomainSystemLaunchd, helperName as CFString, authRef, &error) {
            // TODO: throw
            print("SMJobBless error: \(String(describing: error))")
            return false
        }

        if !isHelperAuthorized() {
            SMAppService.openSystemSettingsLoginItems()
        }
        return true
    }

    func isHelperAuthorized() -> Bool {
        if let url = URL(string: "/Library/LaunchDaemons/\(helperName).plist"),
           SMAppService.statusForLegacyPlist(at: url) == .enabled {
            return true
        }
        return false
    }

    func isHelperRunning() -> Bool {
        guard let output = Shell.exec(command: Command.isHelperRunning), !output.isEmpty
        else {
            return false
        }
        return true
    }
}
