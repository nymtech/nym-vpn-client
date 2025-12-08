import SwiftUI
import AppSettings
import Theme
import IssueReporting

public struct CustomNavBar: View {
    private let title: String?
    private let useElevationBackground: Bool
    private let isLogoImageHidden: Bool
    @State private var leftButton: CustomNavBarButton?
    @State private var rightButtons: [CustomNavBarButton]

    @EnvironmentObject private var appSettings: AppSettings

    public init(
        title: String? = nil,
        useElevationBackground: Bool = false,
        isLogoImageHidden: Bool = false,
        leftButton: CustomNavBarButton? = CustomNavBarButton(type: .empty, action: {}),
        rightButtons: [CustomNavBarButton] = []
    ) {
        self.title = title
        self.useElevationBackground = useElevationBackground
        self.isLogoImageHidden = isLogoImageHidden
        _leftButton = State(initialValue: leftButton)
        #if os(macOS)
        _rightButtons = State(initialValue: rightButtons)
        #else
        if rightButtons.count > 1 {
            reportIssue("Multiple right buttons are supported on macOS only. The first one will be used.")
        }
        _rightButtons = State(initialValue: rightButtons.first.map { [$0] } ?? [])
        #endif
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
            CustomNavBarButton(type: .empty, action: nil)
                .accessibilityHidden(true)
        }
        .frame(height: appSettings.isSmallScreen ? 48 : 64)
        .background {
            backgroundColor()
                .ignoresSafeArea()
        }
        .overlay(alignment: .trailing) {
            HStack(spacing: 0) {
                ForEach(Array(rightButtons), id: \.type) { $0 }
            }
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
