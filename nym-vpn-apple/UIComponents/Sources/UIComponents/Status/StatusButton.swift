import SwiftUI
import AppSettings
import Theme

public struct StatusButton: View {
    @EnvironmentObject private var appSettings: AppSettings

    private let config: StatusButtonConfig

    public init(config: StatusButtonConfig) {
        self.config = config
    }

    public var body: some View {
        HStack(alignment: .center, spacing: 10) {
            Text(config.title)
                .foregroundStyle(config.textColor)
                .textStyle(.Headline.Small.regular)
        }
        .padding(.horizontal, appSettings.isSmallScreen ? 20 : 24)
        .padding(.vertical, appSettings.isSmallScreen ? 12 : 16)
        .background(config.backgroundColor)
        .transition(.opacity)
        .animation(.easeInOut, value: config.backgroundColor)
        .cornerRadius(50)
    }
}
