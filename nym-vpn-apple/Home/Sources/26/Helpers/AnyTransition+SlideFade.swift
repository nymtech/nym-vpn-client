import SwiftUI

extension AnyTransition {
    static func slideFade(from edge: Edge) -> AnyTransition {
        let move = AnyTransition.move(edge: edge).combined(with: .opacity)
        return .asymmetric(insertion: move, removal: move)
    }
}
