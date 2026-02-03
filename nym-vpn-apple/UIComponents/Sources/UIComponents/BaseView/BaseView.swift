import SwiftUI
import Theme

public struct BaseView<Content: View>: View {
    private let pageTitleKey: String
    private let leftNavButton: CustomNavBarButton?
    private let rightNavButton: CustomNavBarButton?
    private let content: Content

    public var body: some View {
        VStack(spacing: 0) {
            CustomNavBar(
                title: pageTitleKey.localizedString,
                leftButton: leftNavButton,
                rightButton: rightNavButton
            )

            ScrollView {
                content
            }
            .padding(.horizontal, 16)
            .scrollIndicators(.hidden)
            .frame(maxWidth: MagicNumbers.maxWidth)
        }
        .navigationBarBackButtonHidden(true)
        .frame(maxWidth: .infinity, maxHeight: .infinity)
        .ignoresSafeArea(edges: [.bottom])
        .background {
            NymColor.background
                .ignoresSafeArea()
        }
    }

    public init(
        pageTitleKey: String,
        leftNavButton: CustomNavBarButton? = CustomNavBarButton(type: .empty, action: {}),
        rightNavButton: CustomNavBarButton? = CustomNavBarButton(type: .empty, action: {}),
        @ViewBuilder content: () -> Content
    ) {
        self.pageTitleKey = pageTitleKey
        self.leftNavButton = leftNavButton
        self.rightNavButton = rightNavButton
        self.content = content()
    }
}
