import SwiftUI

public struct GenericImage: View {
    private let imageName: String?
    private let systemImageName: String?

    public init(imageName: String) {
        self.imageName = imageName
        self.systemImageName = nil
    }

    public init(systemImageName: String) {
        self.imageName = nil
        self.systemImageName = systemImageName
    }

    public var body: some View {
        if let imageName {
            Image(imageName, bundle: .module)
                .resizable()
        }
        if let systemImageName {
            Image(systemName: systemImageName)
                .resizable()
                .scaledToFit()
        }
    }
}
