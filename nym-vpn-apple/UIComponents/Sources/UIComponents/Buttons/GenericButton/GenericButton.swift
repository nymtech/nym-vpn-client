import SwiftUI
import Theme

public struct GenericButton: View {
    private let title: String
    private let borderOnly: Bool

    public init(title: String, borderOnly: Bool = false) {
        self.title = title
        self.borderOnly = borderOnly
    }

    public var body: some View {
        HStack {
            Text(title)
                .foregroundStyle(borderOnly ? NymColor.primaryOrange : NymColor.connectTitle)
                .textStyle(.Label.Huge.bold)
        }
        .frame(maxWidth: .infinity, minHeight: 56, maxHeight: 56)
        .background(borderOnly ? .clear : NymColor.primaryOrange)
        .overlay(
            RoundedRectangle(cornerRadius: 8)
                .stroke(NymColor.primaryOrange, lineWidth: borderOnly ? 1 : 0)
        )
        .contentShape(RoundedRectangle(cornerRadius: 8))
        .cornerRadius(8)
    }
}
