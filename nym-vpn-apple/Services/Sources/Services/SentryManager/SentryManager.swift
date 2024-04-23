import Combine
import Sentry
import AppSettings

public final class SentryManager {
    private var appSettings: AppSettings
    private var cancellables = Set<AnyCancellable>()

    public static let shared = SentryManager()

    public init(appSettings: AppSettings = AppSettings.shared) {
        self.appSettings = appSettings
        SentrySDK.start { _ in }
    }

    public func setup() {
        configureSentry()
        setupObservers()
    }
}

extension SentryManager {
    func setupObservers() {
        appSettings.$isErrorReportingOnPublisher.sink { [weak self] _ in
            self?.configureSentry()
        }
        .store(in: &cancellables)
    }

    func configureSentry() {
        Task { @MainActor in
            if appSettings.isErrorReportingOn {
                SentrySDK.start { options in
                    options.dsn = "https://f860c307259ffe7827fa4ecdfaa8834f@o967446.ingest.us.sentry.io/4507135758237696"
                    options.debug = true // Enabled debug when first installing is always helpful
                    options.enableTracing = true

                    // Uncomment the following lines to add more data to your events
                    // options.attachScreenshot = true // This adds a screenshot to the error events
                    // options.attachViewHierarchy = true // This adds the view hierarchy to the error events
                }
            } else {
                SentrySDK.close()
            }
        }
    }
}
