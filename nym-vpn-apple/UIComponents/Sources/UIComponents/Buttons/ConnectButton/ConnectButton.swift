import SwiftUI
import Theme

public struct ConnectButton: View {
    private let state: ConnectButtonState

    @State private var isHovered = false

    public init(state: ConnectButtonState) {
        self.state = state
    }

    public var body: some View {
        HStack {
            Text(state.localizedTitle)
                .foregroundStyle(NymColor.connectTitle)
                .textStyle(.LabelLegacy.Huge.bold)
                .transaction { transaction in
                    transaction.animation = nil
                }
        }
        .frame(maxWidth: .infinity, minHeight: 56, maxHeight: 56)
        .onHover { newValue in
            isHovered = newValue
        }
        .background {
            state.backgroundColor.opacity(isHovered ? 0.7 : 1)
        }
        .cornerRadius(8)
    }
}
