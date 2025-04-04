import SwiftUI
import Theme

public struct BoltImage: View {
    public init() {}

    public var body: some View {
        Image("bolt", bundle: .module)
            .resizable()
            .frame(width: 24, height: 24)
            .cornerRadius(50)
            .foregroundStyle(NymColor.primary)
    }
}
