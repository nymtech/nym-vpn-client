import SwiftUI
import AppSettings

public struct SnackbarView: View {
    private let appSettings: AppSettings
    private let style: SnackbarStyle
    private let message: String

    @Binding private var isDisplayed: Bool

    public init(
        isDisplayed: Binding<Bool>,
        style: SnackbarStyle,
        message: String,
        appSettings: AppSettings = AppSettings.shared
    ) {
        self._isDisplayed = isDisplayed
        self.style = style
        self.message = message
        self.appSettings = appSettings
    }

    public var body: some View {
        VStack {
            if isDisplayed {
                HStack(alignment: .center, spacing: 12) {
                    if let name = style.systemIconName {
                        Image(systemName: name)
                            .resizable()
                            .foregroundColor(style.iconColor)
                            .aspectRatio(contentMode: .fit)
                            .frame(width: 14, height: 14)
                    }

                    Text(message)
                        .foregroundColor(style.textColor)
                        .font(.system(size: 14))
                        .frame(alignment: .leading)
                }
                .frame(maxWidth: .infinity, alignment: .leading)
                .padding(.horizontal, 16)
                .frame(maxWidth: .infinity, minHeight: 35)
                .padding(.vertical, 8)
                .background(style.backgroundColor)
                .cornerRadius(10)
                .padding(.horizontal, 16)
                .padding(.top, appSettings.isSmallScreen ? 64 : 80) // CustomNavBarSize + 16
                .transition(.move(edge: .top))
                .animation(.easeInOut, value: isDisplayed)
            }
            Spacer()
        }
        .onAppear {
            Task {
                while true {
                    try? await Task.sleep(for: .seconds(3))
                    withAnimation {
                        isDisplayed = false
                    }
                    try? await Task.sleep(for: .seconds(3))
                    withAnimation {
                        isDisplayed = true
                    }
                }
            }
        }
    }
}
