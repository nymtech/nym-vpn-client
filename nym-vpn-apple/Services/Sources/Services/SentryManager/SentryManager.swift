import Combine
import Sentry
import AppSettings

@MainActor public final class SentryManager {
    private var appSettings: AppSettings
    private var cancellables = Set<AnyCancellable>()

    public static let shared = SentryManager(appSettings: .shared)

    public init(appSettings: AppSettings) {
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
        if appSettings.isErrorReportingOn {
            SentrySDK.start { options in
                options.dsn = "https://f860c307259ffe7827fa4ecdfaa8834f@o967446.ingest.us.sentry.io/4507135758237696"
                options.debug = false
                options.tracesSampleRate = 1.0

                // Uncomment the following lines to add more data to your events
                // options.attachScreenshot = true // This adds a screenshot to the error events
                // options.attachViewHierarchy = true // This adds the view hierarchy to the error events
            }
        } else {
            SentrySDK.close()
        }
    }
}
