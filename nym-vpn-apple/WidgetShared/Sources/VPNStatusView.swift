import SwiftUI
import WidgetKit
import AppIntents
import NetworkExtension

public struct VPNStatusView: View {
    public let entry: VPNStatusTimelineEntry

    public init(entry: VPNStatusTimelineEntry) {
        self.entry = entry
    }

    public var body: some View {
        VStack(spacing: 0) {
            topSection
            statusLabel
            connectButton
        }
        .padding(12)
        .containerBackground(for: .widget) {
            NymWidgetColors.background
        }
    }
}

private extension VPNStatusView {
    var topSection: some View {
        HStack(alignment: .center, spacing: 0) {
            statusIcon
                .frame(maxHeight: .infinity)

            Spacer(minLength: 8)

            if showLocations {
                locationLabels
                Spacer(minLength: 0)
            }
        }
    }

    var locationLabels: some View {
        VStack(alignment: .leading, spacing: 2) {
            Text("Entry")
                .font(.caption2)
                .foregroundStyle(.secondary)
            Text(entry.entryLocation)
                .font(.caption.weight(.medium))
                .minimumScaleFactor(0.6)
                .lineLimit(1)
            Text("Exit")
                .font(.caption2)
                .foregroundStyle(.secondary)
                .padding(.top, 2)
            Text(entry.exitLocation)
                .font(.caption.weight(.medium))
                .minimumScaleFactor(0.6)
                .lineLimit(1)
        }
    }

    var statusLabel: some View {
        statusText
            .font(.headline)
            .minimumScaleFactor(0.6)
            .lineLimit(1)
            .frame(maxWidth: .infinity, alignment: .leading)
    }

    var connectButton: some View {
        Button(intent: ToggleVPNIntent()) {
            Text(buttonTitle)
                .font(.caption.weight(.semibold))
                .minimumScaleFactor(0.6)
                .lineLimit(1)
                .frame(maxWidth: .infinity)
        }
        .buttonStyle(.borderedProminent)
        .tint(buttonTint)
        .padding(.top, 4)
    }

    var showLocations: Bool {
        entry.status.isConnected && !entry.entryLocation.isEmpty && !entry.exitLocation.isEmpty
    }

    var statusIcon: some View {
        Image(statusImageName)
            .resizable()
            .scaledToFit()
            .frame(width: 60, height: 60)
            .foregroundStyle(iconColor)
    }

    var iconColor: Color {
        switch entry.status {
        case .error:
            return NymWidgetColors.error
        case .status(let neStatus):
            switch neStatus {
            case .connected, .reasserting:
                return NymWidgetColors.accent
            default:
                return .secondary
            }
        case .notConfigured:
            return .secondary
        }
    }

    var statusImageName: String {
        switch entry.status {
        case .error:
            return "menubarError"
        case .status(let neStatus):
            switch neStatus {
            case .connected, .reasserting:
                return "menubarConnected"
            case .connecting:
                return "menubarConnecting"
            default:
                return "menubarDisconnected"
            }
        case .notConfigured:
            return "menubarDisconnected"
        }
    }

    var statusText: some View {
        Group {
            if entry.status.isConnecting {
                Text("Connecting...")
            } else if entry.status.isDisconnecting {
                Text("Disconnecting...")
            } else if entry.status.isConnected {
                Text("Connected")
                    .foregroundStyle(NymWidgetColors.accent)
            } else {
                switch entry.status {
                case .error:
                    Text("Error")
                        .foregroundStyle(NymWidgetColors.error)
                default:
                    Text("Disconnected")
                }
            }
        }
    }

    var buttonTitle: String {
        if entry.status.isConnected || entry.status.isConnecting {
            return "Disconnect"
        } else {
            return "Connect"
        }
    }

    var buttonTint: Color {
        if entry.status.isConnected || entry.status.isConnecting {
            return NymWidgetColors.error
        }
        return NymWidgetColors.accent
    }
}
