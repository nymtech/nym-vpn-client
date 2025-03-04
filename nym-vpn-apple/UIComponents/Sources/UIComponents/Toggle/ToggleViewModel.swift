import SwiftUI
import Theme

public final class ToggleViewModel: ObservableObject, Identifiable, Hashable {
    public let id = UUID()

    @Published var isOn = false {
        didSet {
            configure(with: isOn)
        }
    }
    @Published var offset = CGFloat(0)
    @Published var circleDiameter = CGFloat(16)
    @Published var circleColor = NymColor.toggleStroke
    @Published var backgroundColor = NymColor.toggleBackground
    @Published var strokeColor = NymColor.toggleStroke

    private var action: ((Bool) -> Void)

    public init(isOn: Bool, action: @escaping ((Bool) -> Void)) {
        self.isOn = isOn
        self.action = action
        configure(with: isOn)
    }

    public func hash(into hasher: inout Hasher) {
        hasher.combine(id)
    }

    public static func == (lhs: ToggleViewModel, rhs: ToggleViewModel) -> Bool {
        lhs.id == rhs.id
    }
}

extension ToggleViewModel {
    func onTap() {
        isOn.toggle()
        action(isOn)
    }
}

private extension ToggleViewModel {
    func configure(with isOn: Bool) {
        offset.negate()
        offset = isOn ? 8 : -8
        circleDiameter = isOn ? 24 : 16
        circleColor = isOn ? NymColor.sysOnPrimary : NymColor.toggleStroke
        backgroundColor = isOn ? NymColor.accent : NymColor.toggleBackground
        strokeColor = isOn ? NymColor.accent : NymColor.toggleStroke
    }
}
