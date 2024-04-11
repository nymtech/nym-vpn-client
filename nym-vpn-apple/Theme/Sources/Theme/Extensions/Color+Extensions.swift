import SwiftUI

extension Color {
    init(_ name: String) {
        #if os(iOS)
        guard let namedColor = UIColor(named: name, in: Bundle.module, compatibleWith: nil)
        else {
            fatalError("Could not load color from Theme module")
        }
        #else
        guard let namedColor = NSColor(named: name, bundle: Bundle.module) else {
            fatalError("Could not load color from Theme module")
        }
        #endif

        self.init(namedColor)
    }
}
