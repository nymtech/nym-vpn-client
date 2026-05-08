import SwiftUI
import Theme

public struct SelectionMarker: View {
    public init() {}

    public var body: some View {
        UnevenRoundedRectangle(bottomTrailingRadius: 4, topTrailingRadius: 4)
            .foregroundColor(NymColor.accent)
            .frame(width: 4)
    }
}
