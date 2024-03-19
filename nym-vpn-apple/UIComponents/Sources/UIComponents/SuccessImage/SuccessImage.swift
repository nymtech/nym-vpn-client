import SwiftUI

public struct SuccessImage: View {
    public init() {}

    public var body: some View {
        Image("surveySuccess", bundle: .module)
    }
}

#Preview {
    SuccessImage()
}
