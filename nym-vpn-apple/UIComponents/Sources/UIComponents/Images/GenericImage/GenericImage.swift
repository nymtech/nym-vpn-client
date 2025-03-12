import SwiftUI

public struct GenericImage: View {
    private let imageName: String?
    private let systemImageName: String?
    private let allowsHover: Bool

    @State private var isHovered = false

    public init(imageName: String, allowsHover: Bool = false) {
        self.imageName = imageName
        self.systemImageName = nil
        self.allowsHover = allowsHover
    }

    public init(systemImageName: String, allowsHover: Bool = false) {
        self.imageName = nil
        self.systemImageName = systemImageName
        self.allowsHover = allowsHover
    }

    public var body: some View {
        if let imageName {
            Image(imageName, bundle: .module)
                .resizable()
                .scaledToFit()
                .onHover { newValue in
                    guard allowsHover else { return }
                    isHovered = newValue
                }
                .opacity(isHovered ? 0.7 : 1)
        }
        if let systemImageName {
            Image(systemName: systemImageName)
                .resizable()
                .scaledToFit()
                .onHover { newValue in
                    guard allowsHover else { return }
                    isHovered = newValue
                }
                .opacity(isHovered ? 0.7 : 1)
        }
    }
}
