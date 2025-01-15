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

public extension SentryManager {
    func submitFeedback(recommendation: String, message: String) {
        let initialIsErrorReporingOnValue = appSettings.isErrorReportingOn
        appSettings.isErrorReportingOn = true

        Task {
            try await Task.sleep(for: .seconds(3))
            let eventId = SentrySDK.capture(message: "Feedback submission")
            let userFeedback = UserFeedback(eventId: eventId)
            userFeedback.comments = "1. \(recommendation) \n 2.\(message)"
            SentrySDK.capture(userFeedback: userFeedback)

            try await Task.sleep(for: .seconds(4))
            Task { @MainActor in
                appSettings.isErrorReportingOn = initialIsErrorReporingOnValue
            }
        }
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
        Task { @MainActor [weak self] in
            guard let self else { return }
            if appSettings.isErrorReportingOn {
                SentrySDK.start { options in
                    options.dsn = "https://f860c307259ffe7827fa4ecdfaa8834f@o967446.ingest.us.sentry.io/4507135758237696"
                    options.debug = false // Enabled debug when first installing is always helpful
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
