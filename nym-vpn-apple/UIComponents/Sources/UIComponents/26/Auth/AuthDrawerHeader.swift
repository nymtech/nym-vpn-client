import SwiftUI
import Theme

public struct AuthDrawerHeader: View {
    private let showsBackButton: Bool
    private let onBackTapped: () -> Void

    public init(
        showsBackButton: Bool = true,
        onBackTapped: @escaping () -> Void = {}
    ) {
        self.showsBackButton = showsBackButton
        self.onBackTapped = onBackTapped
    }

    public var body: some View {
        ZStack {
            GenericImage(imageName: "logoText")
                .frame(width: 100, height: 27)
            if showsBackButton {
                HStack {
                    NymBackButton(action: onBackTapped)
                    Spacer()
                }
            }
        }
    }
}
