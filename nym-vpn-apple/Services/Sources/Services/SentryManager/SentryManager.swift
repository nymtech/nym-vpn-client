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
        Task { @MainActor [weak self] in
            guard let self else { return }
            if appSettings.isErrorReportingOn {
                SentrySDK.start { options in
                    options.dsn = "https://f860c307259ffe7827fa4ecdfaa8834f@o967446.ingest.us.sentry.io/4507135758237696"
                    options.debug = false
                    options.tracesSampleRate = 1
                }
            } else {
                SentrySDK.close()
            }
        }
    }
}
