import SwiftUI
import AppSettings
import Theme

public struct StatusButton: View {
    @EnvironmentObject private var appSettings: AppSettings

    private let config: StatusButtonConfig
    private let isSmallScreen: Bool

    public init(config: StatusButtonConfig, isSmallScreen: Bool = false) {
        self.config = config
        self.isSmallScreen = isSmallScreen
    }

    public var body: some View {
        HStack(alignment: .center, spacing: 10) {
            Text(config.title)
                .foregroundStyle(config.textColor)
                .textStyle(isSmallScreen ? .LabelLegacy.Large.bold : .LabelLegacy.Huge.bold)
        }
        .padding(.horizontal, appSettings.isSmallScreen ? 20 : 24)
        .padding(.vertical, isSmallScreen ? 12 : 16)
        .background(config.backgroundColor)
        .transition(.opacity)
        .animation(.easeInOut, value: config.backgroundColor)
        .cornerRadius(50)
    }
}
