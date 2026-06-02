import Foundation

@Observable
public final class AppDiscoveryService {
    public init() {}

    public func enumerateApps() -> [FoundApp] {
        let folders = [
            URL(filePath: "/Applications"),
            URL(filePath: "/Applications/Utilities"),
            URL(filePath: "/System/Applications")
        ]
        return folders
            .flatMap { enumerateApps(in: $0) }
            .sorted { lhs, rhs in
                lhs.name.localizedCaseInsensitiveCompare(rhs.name) == .orderedAscending
            }
    }

    public func foundApp(at url: URL) -> FoundApp {
        makeFoundApp(from: url)
    }
}

private extension AppDiscoveryService {
    func enumerateApps(in folder: URL?) -> [FoundApp] {

        let fileManager = FileManager.default

        guard let folder else { return [] }

        guard let enumerator = fileManager.enumerator(
            at: folder,
            includingPropertiesForKeys: nil,
            options: [.skipsSubdirectoryDescendants]
        ) else {
            return []
        }

        var apps = [FoundApp]()

        for case let appURL as URL in enumerator {
            guard appURL.pathExtension == "app" && appURL.lastPathComponent != "NymVPN.app" else { continue }
            apps.append(makeFoundApp(from: appURL))
        }

        return apps
    }

    func makeFoundApp(from appURL: URL) -> FoundApp {
        let name = appURL.deletingPathExtension().lastPathComponent

        let bundle = Bundle(url: appURL)
        let info = bundle?.infoDictionary

        let executablePath = resolveExecutablePath(appURL: appURL, info: info)?.path
        let iconURL = resolveIconURL(bundle: bundle, info: info)

        return FoundApp(
            name: name,
            executablePath: executablePath,
            icon: iconURL
        )
    }

    func resolveExecutablePath(appURL: URL, info: [String: Any]?) -> URL? {
        guard let info,
            let executableName = info["CFBundleExecutable"] as? String
        else {
            return nil
        }

        // macOS app bundle layout: MyApp.app/Contents/MacOS/MyApp
        let macOSExecutableURL = appURL
            .appendingPathComponent("Contents")
            .appendingPathComponent("MacOS")
            .appendingPathComponent(executableName)

        guard FileManager.default.fileExists(atPath: macOSExecutableURL.path) else { return nil }
        return macOSExecutableURL
    }

    func resolveIconURL(bundle: Bundle?, info: [String: Any]?) -> URL? {
        guard let bundle, let info else { return nil }

        // CFBundleIconFile (often without extension)
        if let iconFile = info["CFBundleIconFile"] as? String,
           let url = resolveIconURL(bundle: bundle, iconNameOrFile: iconFile) {
            return url
        }

        // CFBundleIconName
        if let iconName = info["CFBundleIconName"] as? String,
           let url = resolveIconURL(bundle: bundle, iconNameOrFile: iconName) {
            return url
        }

        // CFBundleIcons -> CFBundlePrimaryIcon -> CFBundleIconFiles (array)
        if let icons = info["CFBundleIcons"] as? [String: Any],
           let primary = icons["CFBundlePrimaryIcon"] as? [String: Any],
           let iconFiles = primary["CFBundleIconFiles"] as? [String],
           let first = iconFiles.first,
           let url = resolveIconURL(bundle: bundle, iconNameOrFile: first) {
            return url
        }

        return nil
    }

    func resolveIconURL(bundle: Bundle, iconNameOrFile: String) -> URL? {
        // .icns in Contents/Resources
        let candidates = [
            iconNameOrFile,
            iconNameOrFile.hasSuffix(".icns") ? iconNameOrFile : "\(iconNameOrFile).icns"
        ]

        for candidate in candidates {
            if let url = bundle.url(forResource: candidate, withExtension: nil) {
                return url
            }

            let name = (candidate as NSString).deletingPathExtension
            let ext = (candidate as NSString).pathExtension
            guard ext.isEmpty == false else { continue }

            if let url = bundle.url(forResource: name, withExtension: ext) {
                return url
            }
        }

        return nil
    }
}
