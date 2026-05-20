import SwiftUI
import Theme

public struct PagerView<Page: View>: View {
    private let pageCount: Int
    private let pageSpacing: CGFloat
    private let page: (Int) -> Page

    @State private var ignore: Bool = false
    @GestureState private var offsetX: CGFloat = 0
    @State private var measuredHeight: CGFloat = 0

    @Binding private var currentIndex: Int {
        didSet { if !ignore { currentFloatIndex = CGFloat(currentIndex) } }
    }

    @State private var currentFloatIndex: CGFloat = 0 {
        didSet {
            ignore = true
            currentIndex = min(max(Int(currentFloatIndex.rounded()), 0), pageCount - 1)
            ignore = false
        }
    }

    public init(
        pageCount: Int,
        currentIndex: Binding<Int>,
        pageSpacing: CGFloat = 24,
        @ViewBuilder page: @escaping (Int) -> Page
    ) {
        self.pageCount = pageCount
        self.pageSpacing = pageSpacing
        self.page = page
        _currentIndex = currentIndex
    }

    public var body: some View {
        GeometryReader { geometry in
            let containerWidth = geometry.size.width
            let inset = pageSpacing / 2
            let pageWidth = max(containerWidth - (2 * inset), 1)
            let stride = pageWidth + pageSpacing

            HStack(spacing: pageSpacing) {
                ForEach(0..<pageCount, id: \.self) { index in
                    page(index)
                        .frame(width: pageWidth)
                        .accessibilityHidden(index != currentIndex)
                }
            }
            .padding(.horizontal, inset)
            .frame(width: containerWidth, alignment: .leading)
            .offset(x: -CGFloat(currentFloatIndex) * stride)
            .offset(x: offsetX)
            .animation(.linear, value: offsetX)
            .clipped()
            .onGeometryChange(for: CGFloat.self) { proxy in
                proxy.size.height
            } action: { newHeight in
                if newHeight > 0, abs(measuredHeight - newHeight) > 0.5 {
                    measuredHeight = newHeight
                }
            }
            .highPriorityGesture(
                DragGesture()
                    .updating($offsetX) { value, state, _ in
                        state = value.translation.width
                    }
                    .onEnded { value in
                        let offset = value.translation.width / stride
                        let offsetPredicted = value.predictedEndTranslation.width / stride
                        let newIndex = currentFloatIndex - offset

                        currentFloatIndex = newIndex

                        withAnimation(.easeOut) {
                            if offsetPredicted < -0.5 && offset > -0.5 {
                                currentFloatIndex = CGFloat(clampIndex(Int(newIndex.rounded()) + 1))
                            } else if offsetPredicted > 0.5 && offset < 0.5 {
                                currentFloatIndex = CGFloat(clampIndex(Int(newIndex.rounded()) - 1))
                            } else {
                                currentFloatIndex = CGFloat(clampIndex(Int(newIndex.rounded())))
                            }
                        }
                    }
            )
            .accessibilityElement(children: .combine)
            .accessibilityLabel("onboarding.pager.label".localizedString)
            .accessibilityValue(
                String(
                    format: "onboarding.pager.position".localizedString,
                    currentIndex + 1,
                    pageCount
                )
            )
            .accessibilityAdjustableAction { direction in
                switch direction {
                case .increment:
                    goToPage(currentIndex + 1)
                case .decrement:
                    goToPage(currentIndex - 1)
                @unknown default:
                    break
                }
            }
        }
        .frame(height: measuredHeight == 0 ? nil : measuredHeight)
        .onAppear { currentFloatIndex = CGFloat(currentIndex) }
        .onChange(of: currentIndex) { _, newValue in
            withAnimation(.easeOut) {
                currentFloatIndex = CGFloat(newValue)
            }
        }
    }

    private func goToPage(_ index: Int) {
        let clamped = clampIndex(index)
        withAnimation(.easeOut) {
            currentFloatIndex = CGFloat(clamped)
        }
    }

    private func clampIndex(_ index: Int) -> Int {
        min(max(index, 0), pageCount - 1)
    }
}
