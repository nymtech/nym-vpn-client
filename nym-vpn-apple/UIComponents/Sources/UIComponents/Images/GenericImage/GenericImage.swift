import SwiftUI

public struct GenericImage: View {
    private let imageName: String

    public init(imageName: String) {
        self.imageName = imageName
    }

    public var body: some View {
        Image(imageName, bundle: .module)
            .resizable()
    }
}
