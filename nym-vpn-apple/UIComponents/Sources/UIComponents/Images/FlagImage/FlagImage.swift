import SwiftUI

public struct FlagImage: View {
    private let countryCode: String

    public init(countryCode: String) {
        self.countryCode = countryCode.lowercased()
    }

    public var body: some View {
        Image(countryCode, bundle: .module)
            .resizable()
            .scaledToFit()
            .frame(width: 24, height: 24)
            .cornerRadius(50)
    }
}
