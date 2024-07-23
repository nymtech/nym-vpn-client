import SwiftUI
import AppSettings
import Theme

public struct CustomNavBar: View {
    @EnvironmentObject private var appSettings: AppSettings

    public let title: String?
    public let leftButton: CustomNavBarButton?
    public let rightButton: CustomNavBarButton?

    public init(
        title: String? = nil,
        leftButton: CustomNavBarButton? = CustomNavBarButton(type: .empty, action: {}),
        rightButton: CustomNavBarButton? = CustomNavBarButton(type: .empty, action: {})
    ) {
        self.title = title
        self.leftButton = leftButton
        self.rightButton = rightButton
    }

    public var body: some View {
        HStack {
            leftButton
            Spacer()
            if let title {
                Text(title)
                    .textStyle(.Title.Large.primary)
            } else {
                Image("logoText", bundle: .module)
                    .frame(width: 110, height: 16)
            }
            Spacer()
            rightButton
        }
        .frame(height: appSettings.isSmallScreen ? 48 : 64)
        .background {
            NymColor.navigationBarBackground
                .ignoresSafeArea()
        }
    }
}
