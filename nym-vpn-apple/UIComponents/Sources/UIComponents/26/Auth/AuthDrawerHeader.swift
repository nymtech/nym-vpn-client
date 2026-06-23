import SwiftUI
import Theme

public struct AuthDrawerHeader: View {
    private let onBackTapped: () -> Void

    public init(onBackTapped: @escaping () -> Void) {
        self.onBackTapped = onBackTapped
    }

    public var body: some View {
        ZStack {
            GenericImage(imageName: "logoText")
                .frame(width: 100, height: 27)
            HStack {
                NymBackButton(action: onBackTapped)
                Spacer()
            }
        }
    }
}
