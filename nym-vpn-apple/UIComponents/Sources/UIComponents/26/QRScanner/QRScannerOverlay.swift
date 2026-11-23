#if os(iOS)
import SwiftUI
import Theme

struct QRScannerOverlay: View {
    let invalidMessage: String?
    let isValidating: Bool
    let secondaryAction: QRScanAction?

    var body: some View {
        GeometryReader { geometry in
            let side = min(geometry.size.width, geometry.size.height) * Constants.windowFraction
            let margin = max(0, (geometry.size.height - side) / 2)
            // The scrim covers the safe areas, so its window is shifted to the safe area's centre.
            let windowOffset = (geometry.safeAreaInsets.top - geometry.safeAreaInsets.bottom) / 2
            ZStack(alignment: .top) {
                scrim(side: side, offset: windowOffset)
                VStack(spacing: 0) {
                    Color.clear
                        .frame(height: margin)
                    Color.clear
                        .frame(height: side)
                    messages
                        .frame(height: margin)
                }
                .accessibilityElement(children: .contain)
            }
        }
    }
}

private extension QRScannerOverlay {
    func scrim(side: CGFloat, offset: CGFloat) -> some View {
        ZStack {
            Color.black.opacity(Constants.scrimOpacity)
            RoundedRectangle(cornerRadius: Constants.windowCornerRadius)
                .frame(width: side, height: side)
                .offset(y: offset)
                .blendMode(.destinationOut)
        }
        .compositingGroup()
        .overlay {
            QRViewfinderBrackets(cornerRadius: Constants.windowCornerRadius, length: Constants.bracketLength)
                .stroke(Color.Nym.primary, style: StrokeStyle(lineWidth: Constants.bracketWidth, lineCap: .round))
                .frame(width: side, height: side)
                .offset(y: offset)
        }
        .ignoresSafeArea()
        .accessibilityHidden(true)
    }

    var messages: some View {
        VStack(spacing: NymSpacing.component) {
            // Messages scroll on short screens so the secondary action stays visible.
            ScrollView {
                VStack(spacing: NymSpacing.component) {
                    Text("qrScanner.hint".localizedString)
                        .nymTextStyle(.bodyLarge)
                        .foregroundStyle(Color.white)
                        .multilineTextAlignment(.center)
                    if let invalidMessage {
                        Text(invalidMessage)
                            .nymTextStyle(.bodyDefault)
                            .foregroundStyle(Color.white)
                            .multilineTextAlignment(.center)
                            .padding(.horizontal, NymSpacing.medium)
                            .padding(.vertical, NymSpacing.small)
                            .background(Color.Nym.error, in: RoundedRectangle(cornerRadius: 8))
                            .transition(.opacity)
                    }
                    if isValidating {
                        HStack(spacing: NymSpacing.small) {
                            ProgressView()
                                .tint(Color.white)
                            Text("qrScanner.checking".localizedString)
                                .nymTextStyle(.bodyDefault)
                                .foregroundStyle(Color.white)
                        }
                        .accessibilityElement(children: .combine)
                        .transition(.opacity)
                    }
                }
                .frame(maxWidth: .infinity)
                .padding(.top, NymSpacing.large)
            }
            .scrollIndicators(.never)
            .animation(.easeInOut, value: invalidMessage)
            .animation(.easeInOut, value: isValidating)

            if let secondaryAction {
                // A code being checked can't be overtaken by another.
                Button(action: secondaryAction.action) {
                    Text(secondaryAction.label)
                        .nymTextStyle(.bodyLarge)
                        .foregroundStyle(Color.white)
                        .frame(minHeight: Constants.actionMinHeight)
                        .padding(.horizontal, NymSpacing.component)
                        .contentShape(Rectangle())
                }
                .buttonStyle(.plain)
                .disabled(isValidating)
                .opacity(isValidating ? 0.5 : 1)
            }
        }
        .padding(.horizontal, NymSpacing.large)
        .padding(.bottom, NymSpacing.large)
    }

    enum Constants {
        static let windowFraction: CGFloat = 0.7
        static let windowCornerRadius: CGFloat = 24
        static let bracketLength: CGFloat = 40
        static let bracketWidth: CGFloat = 4
        static let scrimOpacity: Double = 0.6
        static let actionMinHeight: CGFloat = 48
    }
}

struct QRViewfinderBrackets: Shape {
    let cornerRadius: CGFloat
    let length: CGFloat

    func path(in rect: CGRect) -> Path {
        var path = Path()
        let radius = min(cornerRadius, length)
        let corners: [(start: CGPoint, corner: CGPoint, end: CGPoint)] = [
            (CGPoint(x: rect.minX, y: rect.minY + length), CGPoint(x: rect.minX, y: rect.minY), CGPoint(x: rect.minX + length, y: rect.minY)),
            (CGPoint(x: rect.maxX - length, y: rect.minY), CGPoint(x: rect.maxX, y: rect.minY), CGPoint(x: rect.maxX, y: rect.minY + length)),
            (CGPoint(x: rect.maxX, y: rect.maxY - length), CGPoint(x: rect.maxX, y: rect.maxY), CGPoint(x: rect.maxX - length, y: rect.maxY)),
            (CGPoint(x: rect.minX + length, y: rect.maxY), CGPoint(x: rect.minX, y: rect.maxY), CGPoint(x: rect.minX, y: rect.maxY - length))
        ]
        for corner in corners {
            path.move(to: corner.start)
            path.addArc(tangent1End: corner.corner, tangent2End: corner.end, radius: radius)
            path.addLine(to: corner.end)
        }
        return path
    }
}
#endif
