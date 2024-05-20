import SecurityFoundation
import ServiceManagement
import Shell

public final class HelperManager {
    public static let shared = HelperManager()

    private var helperName = ""

    public func setup(helperName: String) {
        self.helperName = helperName
    }

    public func authorizeAndInstallHelper() -> Bool {
        var authRef: AuthorizationRef?
        let status = AuthorizationCreate(nil, nil, [], &authRef)
        if status != errAuthorizationSuccess {
            return false
        }

        let rightName = kSMRightBlessPrivilegedHelper

        return rightName.withCString { cStringName -> Bool in
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
                    return installHelper(with: authRef)
                }
                return false
            }
        }
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
    func installHelper(with authRef: AuthorizationRef?) -> Bool {
        var cfError: Unmanaged<CFError>?
        // TODO: refactor using SMAPPService
        if !SMJobBless(kSMDomainSystemLaunchd, helperName as CFString, authRef, &cfError) {
            print("SMJobBless error: \(String(describing: cfError))")
            return false
        }
        if !isHelperAuthorized() {
            SMAppService.openSystemSettingsLoginItems()
        }
        return true
    }
}
