import Foundation
#if os(iOS)
import UIKit
#endif

@MainActor
protocol AppIconChanging {
    var currentAlternateIconName: String? { get }
    func setAlternateIconName(_ name: String?) async throws
}

#if os(iOS)
@MainActor
struct UIApplicationAppIconChanger: AppIconChanging {
    var currentAlternateIconName: String? {
        UIApplication.shared.alternateIconName
    }

    func setAlternateIconName(_ name: String?) async throws {
        guard UIApplication.shared.supportsAlternateIcons else { return }
        try await UIApplication.shared.setAlternateIconName(name)
    }
}
#endif
