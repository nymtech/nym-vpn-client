import SwiftUI
import ConnectionTypes
import UIComponents

public struct GatewayRandomCell: View {
    private let hopType: HopType
    @Binding private var path: NavigationPath
    @Binding private var entryGateway: EntryGateway
    @Binding private var exitRouter: ExitRouter
    private let onTapOverride: (() -> Void)?

    private var isSelected: Bool {
        switch hopType {
        case .entry:
            if case .random = entryGateway { return true }
        case .exit:
            if case .random = exitRouter { return true }
        }
        return false
    }

    public init(
        type: HopType,
        path: Binding<NavigationPath>,
        entryGateway: Binding<EntryGateway>,
        exitRouter: Binding<ExitRouter>,
        onTap: (() -> Void)? = nil
    ) {
        self.hopType = type
        _path = path
        _entryGateway = entryGateway
        _exitRouter = exitRouter
        self.onTapOverride = onTap
    }

    public var body: some View {
        GatewaySelectionCell(
            imageName: "random",
            titleKey: "gatewaysView.random",
            isSelected: isSelected,
            onTap: tapAction
        )
    }
}

private extension GatewayRandomCell {
    func tapAction() {
        if let onTapOverride {
            onTapOverride()
        } else {
            switch hopType {
            case .entry:
                entryGateway = .random
            case .exit:
                exitRouter = .random
            }
        }
        path = .init()
    }
}
