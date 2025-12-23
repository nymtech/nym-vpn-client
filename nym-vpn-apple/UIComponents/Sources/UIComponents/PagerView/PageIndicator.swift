import SwiftUI

public struct PageIndicator: View {
    let pageCount: Int
    @Binding var selection: Int

    public init(pageCount: Int, selection: Binding<Int>) {
        self.pageCount = pageCount
        _selection = selection
    }

    public var body: some View {
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
    }
}
