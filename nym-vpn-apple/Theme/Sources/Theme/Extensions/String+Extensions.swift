import Foundation

public extension String {
    var localizedString: String {
        Bundle.main.localizedString(forKey: self)
    }

    var localizedFromMainApp: String {
        let pluginsURL = Bundle.main.bundleURL
            .deletingLastPathComponent() // Plugins
            .deletingLastPathComponent() // app

        guard pluginsURL.pathExtension == "app",
              let mainAppBundle = Bundle(url: pluginsURL)
        else {
            return self
        }

        return mainAppBundle.localizedString(forKey: self)
    }
}
