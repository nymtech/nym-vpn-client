import SecurityFoundation
import ServiceManagement
import Shell

// Any changes made to Info.plist & Launchd.plist - are used to create daemon in nym-vpnd.

public final class HelperManager {
    public static let shared = HelperManager()

    private var helperName = ""

    public func setup(helperName: String) {
        self.helperName = helperName
    }

    // TODO: throw some errors
    // TODO: add completion block on success
    public func authorizeAndInstallHelper() throws -> Bool {
        var authRef: AuthorizationRef?
        let status = AuthorizationCreate(nil, nil, [], &authRef)
        if status != errAuthorizationSuccess {
            return false
        }

        var cfError: Unmanaged<CFError>?

        let rightName = kSMRightBlessPrivilegedHelper

        if let authRef = authRef {

        }
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
                let status = AuthorizationCopyRights(authRef!, &authRights, nil, authFlags, nil)
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

    public func isHelperAuthorized() -> Bool {
        if let url = URL(string: "/Library/LaunchDaemons/\(helperName).plist"),
           SMAppService.statusForLegacyPlist(at: url) == .enabled {
            return true
        }
        return false
    }

    public func isHelperRunning() -> Bool {
        if let output = Shell.exec(command: Command.isHelperRunning), !output.isEmpty {
            return true
        }
        return false
    }
}

private extension HelperManager {
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
}
