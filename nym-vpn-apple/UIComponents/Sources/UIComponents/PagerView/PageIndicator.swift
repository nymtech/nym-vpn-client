import SwiftUI
import Theme

public struct PageIndicator: View {
    let pageCount: Int
    @Binding var selection: Int

    public init(pageCount: Int, selection: Binding<Int>) {
        self.pageCount = pageCount
        _selection = selection
    }

    public var body: some View {
        HStack(spacing: 0) {
            arrowButton()
                .accessibilityLabel("onboarding.previous".localizedString)
                .accessibilityAction {
                    previousPage()
                }
                .onTapGesture {
                    previousPage()
                }
                .rotationEffect(.degrees(-180))

            Spacer()

            HStack(spacing: 8) {
                ForEach(0..<pageCount, id: \.self) { index in
                    Circle()
                        .frame(width: 8, height: 8)
                        .opacity(selection == index ? 1 : 0.25)
                        .onTapGesture {
                            withAnimation(.easeInOut(duration: 0.2)) {
                                selection = index
                            }
                        }
                }
            }
            .padding(.vertical, 8)
            .padding(.horizontal, 12)
            .background(.ultraThinMaterial, in: Capsule())

            Spacer()

            arrowButton()
                .accessibilityLabel("onboarding.next".localizedString)
                .accessibilityAction {
                    nextPage()
                }
                .onTapGesture {
                    nextPage()
                }
        }
    }
}

private extension PageIndicator {
    func arrowButton() -> some View {
        ZStack {
            Circle()
                .fill(NymColor.backgroundHover)
                .frame(width: 40, height: 40)

            Image("arrowRight", bundle: .module)
                .resizable()
                .frame(width: 24, height: 24)
        }
        .frame(width: 48, height: 48)
        .accessibilityElement(children: .combine)
        .accessibilityAddTraits([.isButton])
    }
}

private extension PageIndicator {
    func nextPage() {
        guard selection + 1 <= pageCount - 1 else { return }
        selection += 1
    }

    func previousPage() {
        guard selection - 1 >= 0 else { return }
        selection -= 1
    }
}
