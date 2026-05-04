#if os(iOS)
import SwiftUI
import WidgetKit

@available(iOS 18.0, macOS 26.0, *)
public struct NymVPNControlWidget: ControlWidget {
    public static let displayName = LocalizedStringResource(stringLiteral: "NymVPN")
    public static let description = LocalizedStringResource(stringLiteral: "View and manage your VPN connection.")

    public init() {}

    public var body: some ControlWidgetConfiguration {
        StaticControlConfiguration(
            kind: "NymVPNControlWidget",
            provider: VPNControlStatusValueProvider()
        ) { status in
            ControlWidgetToggle(
                status.isConnected ? "Connected" : "Disconnected",
                isOn: status.isConnected,
                action: ToggleVPNSetValueIntent()
            ) { isOn in
                if isOn {
                    Label("Connected", image: "nymConnected")
                } else {
                    Label("Disconnected", image: "nymDisconnected")
                }
            }
            .tint(.green)
        }
        .displayName(Self.displayName)
        .description(Self.description)
    }
}
#endif
