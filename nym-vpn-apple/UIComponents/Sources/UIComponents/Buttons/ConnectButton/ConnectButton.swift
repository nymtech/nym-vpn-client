import SwiftUI
import Theme

public struct ConnectButton: View {
    private let state: ConnectButtonState

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
        .background(state.backgroundColor)
        .cornerRadius(8)
    }
}
