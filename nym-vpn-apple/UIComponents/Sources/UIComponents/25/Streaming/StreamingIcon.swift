import SwiftUI
import Theme

public struct StreamingIcon: View {
    public var body: some View {
        GenericImage(systemImageName: "play.rectangle")
            .frame(width: 18, height: 18)
            .foregroundStyle(NymColor.info)
    }

    public init() {}
}
