import SecurityFoundation
import ServiceManagement
import Shell

// Any changes made to Info.plist & Launchd.plist - are used to create daemon in nym-vpnd.

public final class HelperManager {
    public static let shared = HelperManager()
    public let requiredVersion = "1.0.0-rc.14"

    private var helperName = ""

    public func setup(helperName: String) {
        self.helperName = helperName
    }

    public func installHelper() async throws {
        try await authorizeAndInstallHelper()
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
    func authorizeAndInstallHelper() async throws {
        var authRef: AuthorizationRef?
        let status = AuthorizationCreate(nil, nil, [], &authRef)
        guard status == errAuthorizationSuccess, let authRef = authRef
        else {
            throw DaemonError.authorizationDenied
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
        if !result {
            throw DaemonError.authorizationDenied
        }
        if let error = cfError?.takeRetainedValue() {
            throw error
        }
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
