import SwiftUI
import AppSettings
import Theme

public struct CustomNavBar: View {
    private let title: String?
    private let useElevationBackground: Bool
    private let isLogoImageHidden: Bool
    @State private var leftButton: CustomNavBarButton?
    @State private var rightButton: CustomNavBarButton?

    @EnvironmentObject private var appSettings: AppSettings

    public init(
        title: String? = nil,
        useElevationBackground: Bool = false,
        isLogoImageHidden: Bool = false,
        leftButton: CustomNavBarButton? = CustomNavBarButton(type: .empty, action: {}),
        rightButton: CustomNavBarButton? = CustomNavBarButton(type: .empty, action: {})
    ) {
        self.title = title
        self.useElevationBackground = useElevationBackground
        self.isLogoImageHidden = isLogoImageHidden
        self.leftButton = leftButton
        self.rightButton = rightButton
    }

    public var body: some View {
        HStack {
            leftButton
            Spacer()
            if let title {
                Text(title)
                    .textStyle(.Headline.Medium.regular)
            } else if !isLogoImageHidden {
                Image("logoText", bundle: .module)
                    .frame(width: 110, height: 16)
                    .accessibilityLabel("NymVPN".localizedString)
            }
            Spacer()
            rightButton
        }
        .frame(height: appSettings.isSmallScreen ? 48 : 64)
        .background {
            backgroundColor()
                .ignoresSafeArea()
        }
    }
}

private extension CustomNavBar {
    func backgroundColor() -> Color {
        if useElevationBackground {
            return NymColor.background
        } else {
            return NymColor.elevation
        }
    }
}
