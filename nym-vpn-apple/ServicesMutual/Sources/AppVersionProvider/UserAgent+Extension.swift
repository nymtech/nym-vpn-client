#if os(iOS)
import NymVPNLib

extension UserAgent {
    /// User-agent identifying the application
    public static var appUserAgent: UserAgent {
        UserAgent(
            application: AppVersionProvider.app,
            version: "\(AppVersionProvider.appVersion()) (\(AppVersionProvider.realAppVersion()))",
            platform: AppVersionProvider.platform,
            gitCommit: ""
        )
    }
}
#endif
