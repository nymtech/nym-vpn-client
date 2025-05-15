import SwiftUI
import Theme
import UIComponents

public struct LogoView: View {
    public var body: some View {
        VStack {
            Spacer()
            HStack {
                Spacer()
                GenericImage(imageName: "logoText")
                    .frame(width: 120)
                Spacer()
            }
            Spacer()
        }
        .background {
            NymColor.background
                .ignoresSafeArea()
        }
    }

    public init() {}
}
