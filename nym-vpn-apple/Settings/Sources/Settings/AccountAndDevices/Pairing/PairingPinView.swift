#if os(iOS)
import SwiftUI
import Theme
import UIComponents

struct PairingPinView: View {
    let pin: String

    var body: some View {
        Text(groupedPin)
            .font(.system(.largeTitle, design: .monospaced, weight: .semibold))
            .tracking(4)
            .foregroundStyle(Color.Nym.textPrimary)
            .padding(.horizontal, NymSpacing.section)
            .padding(.vertical, NymSpacing.large)
            .background(Color.Nym.surfaceAlt, in: RoundedRectangle(cornerRadius: 12))
            .accessibilityElement()
            .accessibilityLabel(String(format: "pairing.pin.accessibility".localizedString, pin.map(String.init).joined(separator: " ")))
    }

    private var groupedPin: String {
        stride(from: 0, to: pin.count, by: 3)
            .map { start in
                let from = pin.index(pin.startIndex, offsetBy: start)
                let to = pin.index(from, offsetBy: 3, limitedBy: pin.endIndex) ?? pin.endIndex
                return String(pin[from..<to])
            }
            .joined(separator: " ")
    }
}
#endif
