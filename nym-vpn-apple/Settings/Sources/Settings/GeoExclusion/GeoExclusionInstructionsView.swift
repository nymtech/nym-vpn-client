#if os(macOS)
import SwiftUI
import Theme
import UIComponents

public struct GeoExclusionInstructionsView: View {
    @Binding private var path: NavigationPath
    private let listenPort: UInt16

    public init(path: Binding<NavigationPath>, listenPort: UInt16) {
        _path = path
        self.listenPort = listenPort
    }

    private var proxyAddress: String {
        "127.0.0.1:\(listenPort)"
    }

    public var body: some View {
        VStack(spacing: 0) {
            CustomNavBar(
                title: "geoExclusion.setupInstructions".localizedString,
                leftButton: CustomNavBarButton(type: .back) { navigateBack() }
            )

            ScrollView {
                VStack(alignment: .leading, spacing: 16) {
                    Text("geoExclusion.setup.description".localizedString)
                        .foregroundStyle(Color.Nym.textSecondary)
                        .nymTextStyle(.bodyDefault)

                    stepsCard
                    proxyAddressCard
                }
                .padding(.vertical, 24)
                .frame(maxWidth: MagicNumbers.maxWidth)
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

    var stepsCard: some View {
        let steps = [
            "geoExclusion.setup.step1".localizedString,
            "geoExclusion.setup.step2".localizedString,
            "geoExclusion.setup.step3".localizedString
        ]
        return VStack(spacing: 0) {
            ForEach(Array(steps.enumerated()), id: \.offset) { index, step in
                HStack(spacing: 16) {
                    stepCircle(index + 1)
                    Text(step)
                        .foregroundStyle(Color.Nym.textPrimary)
                        .nymTextStyle(.bodyDefault)
                    Spacer(minLength: 0)
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
        VStack(alignment: .leading, spacing: 8) {
            Text("geoExclusion.setup.proxyAddress".localizedString.uppercased())
                .foregroundStyle(Color.Nym.textTertiary)
                .nymTextStyle(.bodySmallBold)
            Text(proxyAddress)
                .foregroundStyle(Color.Nym.textPrimary)
                .font(Font.custom("Courier New", size: 15))
                .kerning(0.3)
        }
        .padding(16)
        .frame(maxWidth: .infinity, alignment: .leading)
        .background(RoundedRectangle(cornerRadius: 12).fill(Color.Nym.surface))
    }
}
#endif
