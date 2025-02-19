import SwiftUI

public struct FlagImage: View {
    private let countryCode: String
    private let width: CGFloat
    private let height: CGFloat

    public init(countryCode: String, width: CGFloat = 24, height: CGFloat = 24) {
        self.countryCode = countryCode.lowercased()
        self.width = width
        self.height = height
    }

    public var body: some View {
        Image(countryCode, bundle: .module)
            .resizable()
            .scaledToFit()
            .frame(width: width, height: height)
            .cornerRadius(50)
    }
}
