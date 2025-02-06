import Foundation

public enum AppVersionProvider {
    public static let libVersion = "1.3.0-dev.20250206031003"

    public static var app: String {
        "nym-vpn-app"
    }

    public static var platform: String {
        "\(osType()); \(osVersion()); \(hardwareString())"
    }

    public static func appVersion(in bundle: Bundle = .main) -> String {
        guard let version = bundle.object(forInfoDictionaryKey: "CFBundleShortVersionString") as? String
        else {
            fatalError("Missing CFBundleShortVersionString")
        }
        return version
    }
}

private extension AppVersionProvider {
    static func osType() -> String {
#if os(OSX)
        return "macOS"
#elseif os(watchOS)
        return "watchOS"
#elseif os(tvOS)
        return "tvOS"
#elseif os(iOS)
#if targetEnvironment(macCatalyst)
        return "macOSCatalyst"
#else
        return "iOS"
#endif
#endif
    }

    static func osVersion() -> String {
        let os = ProcessInfo().operatingSystemVersion
        return "\(os.majorVersion).\(os.minorVersion).\(os.patchVersion)"
    }

    static func hardwareString() -> String {
        var name: [Int32] = [CTL_HW, HW_MACHINE]
        var size: Int = 2
        sysctl(&name, 2, nil, &size, nil, 0)
        var hwMachine = [CChar](repeating: 0, count: Int(size))
        sysctl(&name, 2, &hwMachine, &size, nil, 0)

        var hardware = String(cString: hwMachine)
        let simulatorSet: Set<String> = [
            "arm64",
            "i386",
            "x86_64"
        ]
        if simulatorSet.contains(hardware),
           let deviceID = ProcessInfo.processInfo.environment["SIMULATOR_MODEL_IDENTIFIER"] {
            hardware = deviceID
        }
        return hardware
    }
}
