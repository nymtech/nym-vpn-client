import SwiftUI
import Theme

public struct GenericButton: View {
    private let title: String

    public init(title: String) {
        self.title = title
    }

    public var body: some View {
        HStack {
            Text(title)
                .foregroundStyle(NymColor.connectTitle)
                .textStyle(.Label.Huge.bold)
        }
        .frame(maxWidth: .infinity, minHeight: 56, maxHeight: 56)
        .background(NymColor.primaryOrange)
        .cornerRadius(8)
    }
}
