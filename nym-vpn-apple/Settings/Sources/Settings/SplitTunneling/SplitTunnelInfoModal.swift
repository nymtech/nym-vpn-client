#if os(macOS)
import SwiftUI
import Theme
import UIComponents

struct SplitTunnelInfoModal: View {
    @Binding var isDisplayed: Bool

    var body: some View {
        ModalOverlayView(isDisplayed: $isDisplayed, dismissOnOverlayTap: false) {
            VStack {
                icon()
                title()
                directSection()
                Spacer()
                    .frame(height: 16)
                viaNymVPNSection()
                gotItButton()
            }
            .padding(.horizontal, 24)
        }
    }
}

private extension SplitTunnelInfoModal {
    @ViewBuilder
    func icon() -> some View {
        Spacer()
            .frame(height: 24)

        Image(systemName: "info.circle")
            .frame(width: 24, height: 24)

        Spacer()
            .frame(height: 16)
    }

    @ViewBuilder
    func title() -> some View {
        Text("splitTunnel.info.title".localizedString)
            .textStyle(.Headline.Medium.regular)
            .foregroundStyle(NymColor.primary)

        Spacer()
            .frame(height: 16)

        HStack {
            Text("splitTunnel.info.subtitle".localizedString)
                .foregroundStyle(NymColor.gray1)
                .textStyle(.Body.Medium.regular)
            Spacer()
        }

        Spacer()
            .frame(height: 8)

        HStack {
            Text("splitTunnel.info.fullDiskAccess".localizedString)
                .foregroundStyle(NymColor.gray1)
                .textStyle(.Body.Medium.regular)
            Spacer()
        }

        Spacer()
            .frame(height: 16)
    }

    @ViewBuilder
    func directSection() -> some View {
        HStack(spacing: 8) {
            Image(systemName: "slash.circle.fill")
                .frame(width: 16, height: 16)
                .foregroundStyle(NymColor.gray1)
            Text("splitTunnel.info.direct".localizedString)
                .textStyle(.Body.Medium.regular)
                .foregroundStyle(NymColor.primary)
            Spacer()
        }

        Spacer()
            .frame(height: 4)

        HStack {
            Text("splitTunnel.info.direct.subtitle".localizedString)
                .foregroundStyle(NymColor.gray1)
                .textStyle(.Body.Medium.regular)
            Spacer()
        }
    }

    @ViewBuilder
    func viaNymVPNSection() -> some View {
        HStack(spacing: 8) {
            Image(systemName: "shield.fill")
                .frame(width: 16, height: 16)
                .foregroundStyle(NymColor.gray1)
            Text("splitTunnel.info.viaNymVPN".localizedString)
                .textStyle(.Body.Medium.regular)
                .foregroundStyle(NymColor.primary)
            Spacer()
        }

        Spacer()
            .frame(height: 4)

        HStack {
            Text("splitTunnel.info.viaNymVPN.subtitle".localizedString)
                .foregroundStyle(NymColor.gray1)
                .textStyle(.Body.Medium.regular)
            Spacer()
        }
    }

    @ViewBuilder
    func gotItButton() -> some View {
        GenericButton(title: "splitTunnel.info.gotIt".localizedString)
            .padding(.vertical, 24)
            .onTapGesture {
                isDisplayed.toggle()
            }
    }
}
#endif
