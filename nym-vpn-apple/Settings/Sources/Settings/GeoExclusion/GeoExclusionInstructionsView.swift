#if os(macOS)
import AppKit
import SwiftUI
import Theme
import UIComponents

public struct GeoExclusionInstructionsView: View {
    @Binding private var path: NavigationPath
    @State private var addressCopied = false
    private let listenPort: UInt16

    private static let proxyHost = "127.0.0.1"
    private static let web3RpcURL = "http://127.0.0.1:8545"

    public init(path: Binding<NavigationPath>, listenPort: UInt16) {
        _path = path
        self.listenPort = listenPort
    }

    private var proxyAddress: String {
        "\(Self.proxyHost):\(listenPort)"
    }

    public var body: some View {
        VStack(spacing: 0) {
            CustomNavBar(
                title: "geoExclusion.setupInstructions".localizedString,
                leftButton: CustomNavBarButton(type: .back) { navigateBack() }
            )

            ScrollView {
                VStack(alignment: .leading, spacing: 24) {
                    Text("geoExclusion.setup.description".localizedString)
                        .foregroundStyle(Color.Nym.textSecondary)
                        .nymTextStyle(.bodyDefault)

                    systemWideSection
                    appSection
                    web3Section
                    proxyAddressCard
                }
                .frame(maxWidth: MagicNumbers.maxWidth)
                .padding(.vertical, 24)
            }
            .scrollIndicators(.never)
            .padding(.horizontal, 16)
        }
        .navigationBarBackButtonHidden(true)
        .frame(maxWidth: .infinity, maxHeight: .infinity)
        .ignoresSafeArea(edges: [.bottom])
        .background {
            Color.Nym.background
                .ignoresSafeArea()
        }
    }
}

private extension GeoExclusionInstructionsView {
    func navigateBack() {
        guard !path.isEmpty else { return }
        path.removeLast()
    }

    var systemWideSection: some View {
        section(
            title: "geoExclusion.setup.systemWide.title",
            subtitle: "geoExclusion.setup.systemWide.subtitle",
            steps: [
                plainStep("geoExclusion.setup.systemWide.step1"),
                plainStep("geoExclusion.setup.systemWide.step2"),
                highlightedStep("geoExclusion.setup.systemWide.step3", values: [Self.proxyHost, "\(listenPort)"]),
                plainStep("geoExclusion.setup.systemWide.step4")
            ]
        )
    }

    var appSection: some View {
        section(
            title: "geoExclusion.setup.app.title",
            subtitle: "geoExclusion.setup.app.subtitle",
            steps: [
                plainStep("geoExclusion.setup.app.step1"),
                highlightedStep("geoExclusion.setup.app.step2", values: [Self.proxyHost, "\(listenPort)"]),
                plainStep("geoExclusion.setup.app.step3")
            ]
        )
    }

    var web3Section: some View {
        section(
            title: "geoExclusion.setup.web3.title",
            subtitle: nil,
            steps: [
                highlightedStep("geoExclusion.setup.web3.step1", values: [Self.web3RpcURL])
            ]
        )
    }

    func section(title: String, subtitle: String?, steps: [Text]) -> some View {
        VStack(alignment: .leading, spacing: 8) {
            Text(title.localizedString)
                .foregroundStyle(Color.Nym.textPrimary)
                .nymTextStyle(.titleSection)

            if let subtitle {
                Text(subtitle.localizedString)
                    .foregroundStyle(Color.Nym.textSecondary)
                    .nymTextStyle(.bodyDefault)
            }

            stepsCard(steps)
        }
    }

    func stepsCard(_ steps: [Text]) -> some View {
        VStack(spacing: 0) {
            ForEach(Array(steps.enumerated()), id: \.offset) { index, step in
                HStack(spacing: 16) {
                    stepCircle(index + 1)
                    step
                        .nymTextStyle(.bodyDefault)
                        .frame(maxWidth: .infinity, alignment: .leading)
                }
                .padding(16)

                if index < steps.count - 1 {
                    Divider()
                        .frame(height: 1)
                        .overlay(Color.Nym.divider)
                }
            }
        }
        .frame(maxWidth: .infinity)
        .background(RoundedRectangle(cornerRadius: 12).fill(Color.Nym.surface))
    }

    func stepCircle(_ number: Int) -> some View {
        ZStack {
            Circle()
                .stroke(Color.Nym.primary, lineWidth: 1)
                .frame(width: 32, height: 32)
            Text("\(number)")
                .foregroundStyle(Color.Nym.primary)
                .nymTextStyle(.bodySmallBold)
        }
    }

    var proxyAddressCard: some View {
        Button(action: copyProxyAddress) {
            VStack(alignment: .leading, spacing: 8) {
                Text("geoExclusion.setup.proxyAddress".localizedString.uppercased())
                    .foregroundStyle(Color.Nym.textTertiary)
                    .nymTextStyle(.bodySmallBold)
                HStack(spacing: 12) {
                    Text(proxyAddress)
                        .foregroundStyle(Color.Nym.textPrimary)
                        .font(Font.custom("Courier New", size: 15))
                        .kerning(0.3)
                    GenericImage(imageName: addressCopied ? "checkmarkSeeThrough" : "copy")
                        .frame(width: 24, height: 24)
                    Spacer(minLength: 0)
                }
            }
            .padding(16)
            .frame(maxWidth: .infinity, alignment: .leading)
            .contentShape(Rectangle())
        }
        .buttonStyle(.plain)
        .background(RoundedRectangle(cornerRadius: 12).fill(Color.Nym.surface))
    }

    func copyProxyAddress() {
        NSPasteboard.general.prepareForNewContents()
        NSPasteboard.general.setString(proxyAddress, forType: .string)
        withAnimation {
            guard !addressCopied else { return }
            addressCopied = true
            Task { @MainActor in
                try? await Task.sleep(for: .seconds(3))
                addressCopied = false
            }
        }
    }

    /// A step whose whole text is plain body copy.
    func plainStep(_ key: String) -> Text {
        Text(key.localizedString)
            .foregroundColor(Color.Nym.textPrimary)
    }

    /// A step whose `%@` placeholders are filled with green monospace values (IP / port / URL).
    func highlightedStep(_ key: String, values: [String]) -> Text {
        let mono = Font.custom("Courier New", size: 14)
        let parts = key.localizedString.components(separatedBy: "%@")
        var result = Text(verbatim: "")
        for (index, part) in parts.enumerated() {
            result = result + Text(verbatim: part)
                .font(.Nym.bodyDefault)
                .foregroundColor(Color.Nym.textPrimary)
            if index < values.count {
                result = result + Text(verbatim: values[index])
                    .font(mono)
                    .foregroundColor(Color.Nym.primary)
            }
        }
        return result
    }
}
#endif
