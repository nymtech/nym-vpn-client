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
    @Published var circleColor = NymColor.gray1
    @Published var backgroundColor = NymColor.elevation
    @Published var strokeColor = NymColor.gray1
    @Published var isDisabled: Bool

    private var action: ((Bool) -> Void)

    public init(isOn: Bool, isDisabled: Bool = false, action: @escaping ((Bool) -> Void) = { _ in }) {
        self.isOn = isOn
        self.action = action
        self.isDisabled = isDisabled
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
        circleColor = isOn ? NymColor.background : NymColor.gray1
        backgroundColor = isOn ? NymColor.accent : NymColor.elevation
        strokeColor = isOn ? NymColor.accent : NymColor.gray1
    }
}
