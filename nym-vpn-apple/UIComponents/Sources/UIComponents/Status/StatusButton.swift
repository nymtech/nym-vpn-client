import SwiftUI
import Theme

public struct StatusButton: View {
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
                .textStyle(isSmallScreen ? .Label.Large.primary : .Label.Huge.primary)
        }
        .padding(.horizontal, isSmallScreen ? 20 : 24)
        .padding(.vertical, isSmallScreen ? 12 : 16)
        .background(config.backgroundColor)
        .cornerRadius(50)
    }
}
