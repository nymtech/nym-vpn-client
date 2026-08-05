import SwiftUI
import ConnectionTypes
import UIComponents

public struct GatewaySafestCell: View {
    private let hopType: HopType
    @Binding private var path: NavigationPath
    @Binding private var entryGateway: EntryGateway
    @Binding private var exitRouter: ExitRouter
    private let onTapOverride: (() -> Void)?

    private var isSelected: Bool {
        switch hopType {
        case .entry:
            if case .auto = entryGateway { return true }
        case .exit:
            if case .auto = exitRouter { return true }
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
            imageName: "safest",
            titleKey: "gatewaysView.safest",
            isSelected: isSelected,
            onTap: tapAction
        )
    }
}

private extension GatewaySafestCell {
    func tapAction() {
        if let onTapOverride {
            onTapOverride()
        } else {
            switch hopType {
            case .entry:
                entryGateway = .auto
            case .exit:
                exitRouter = .auto
            }
        }
        path = .init()
    }
}
