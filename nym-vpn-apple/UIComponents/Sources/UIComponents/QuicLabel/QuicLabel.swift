import SwiftUI
import Theme

public struct QuicLabel: View {
    public var body: some View {
        HStack(alignment: .center, spacing: 0) {
            GenericImage(systemImageName: "shippingbox")
                .frame(width: 12, height: 12)
                .foregroundStyle(NymColor.quic)
            Spacer()
                .frame(width: 2)
            Text("QUIC")
                .foregroundStyle(NymColor.quic)
                .textStyle(.Body.Small.bold)
        }
        .padding(EdgeInsets(top: 2, leading: 6, bottom: 2, trailing: 6))
        .cornerRadius(4)
        .overlay(
            RoundedRectangle(cornerRadius: 4)
                .inset(by: 0.5)
                .stroke(NymColor.quic.opacity(0.5), lineWidth: 1)
        )
    }

    public init() {}
}
