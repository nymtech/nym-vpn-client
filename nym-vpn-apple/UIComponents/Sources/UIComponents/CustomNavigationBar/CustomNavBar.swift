import SwiftUI
import Theme

public struct CustomNavBar: View {
    public let title: String
    public let leftButton: CustomNavBarButton?
    public let rightButton: CustomNavBarButton?
    public let isSmallScreen: Bool

    public init(
        title: String,
        leftButton: CustomNavBarButton? = CustomNavBarButton(type: .empty, action: {}),
        rightButton: CustomNavBarButton? = CustomNavBarButton(type: .empty, action: {}),
        isSmallScreen: Bool = false
    ) {
        self.title = title
        self.leftButton = leftButton
        self.rightButton = rightButton
        self.isSmallScreen = isSmallScreen
    }

    public var body: some View {
        HStack {
            leftButton
            Spacer()
            Text(title)
                .foregroundStyle(NymColor.sysOnSurface)
                .textStyle(.Title.Large.primary)
            Spacer()
            rightButton
        }
        .frame(height: isSmallScreen ? 48 : 64)
        .background {
            NymColor.navigationBarBackground
                .ignoresSafeArea()
        }
    }
}
