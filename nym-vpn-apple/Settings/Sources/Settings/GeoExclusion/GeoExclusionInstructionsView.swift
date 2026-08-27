#if os(macOS)
import SwiftUI
import Theme
import UIComponents

public struct GeoExclusionInstructionsView: View {
    @Binding private var path: NavigationPath
    private let listenPort: UInt16

    private static let proxyHost = "127.0.0.1"

    public init(path: Binding<NavigationPath>, listenPort: UInt16) {
        _path = path
        self.listenPort = listenPort
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

                    appStepsCard
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

    var appStepsCard: some View {
        stepsCard([
            plainStep("geoExclusion.setup.app.step1"),
            highlightedStep("geoExclusion.setup.app.step2", values: [Self.proxyHost, "\(listenPort)"]),
            plainStep("geoExclusion.setup.app.step3")
        ])
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

    /// A step whose whole text is plain body copy.
    func plainStep(_ key: String) -> Text {
        Text(key.localizedString)
            .foregroundColor(Color.Nym.textPrimary)
    }

    /// A step whose `%@` placeholders are filled with green monospace values (IP / port / URL).
    func highlightedStep(_ key: String, values: [String]) -> Text {
        let mono = Font.custom("Courier New", size: 14)
        let parts = key.localizedString.components(separatedBy: "%@")
        var pieces: [Text] = []
        for (index, part) in parts.enumerated() {
            pieces.append(
                Text(verbatim: part)
                    .font(.Nym.bodyDefault)
                    .foregroundColor(Color.Nym.textPrimary)
            )
            if index < values.count {
                pieces.append(
                    Text(verbatim: values[index])
                        .font(mono)
                        .foregroundColor(Color.Nym.primary)
                )
            }
        }
        return pieces.reduce(Text(verbatim: ""), +)
    }
}
#endif
