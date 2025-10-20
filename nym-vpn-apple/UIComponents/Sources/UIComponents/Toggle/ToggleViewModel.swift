import SwiftUI
import Theme

public final class ToggleViewModel: ObservableObject, Identifiable, Hashable {
    public let controlInAlert: Bool
    public let id = UUID()

    @Binding var isOn: Bool {
        didSet {
            configure(with: isOn)
        }
    }
    @Binding var isDisplayingAlert: Bool
    @Published var offset = CGFloat(0)
    @Published var circleDiameter = CGFloat(16)
    @Published var circleColor = NymColor.gray1
    @Published var backgroundColor = NymColor.elevation
    @Published var strokeColor = NymColor.gray1
    @Published var isDisabled: Bool

    private var action: ((Bool) -> Void)?

    public init(
        isOn: Binding<Bool>,
        controlInAlert: Bool = false,
        isDisplayingAlert: Binding<Bool> = .constant(false),
        isDisabled: Bool = false,
        action: ((Bool) -> Void)? = nil
    ) {
        _isOn = isOn
        _isDisplayingAlert = isDisplayingAlert
        self.controlInAlert = controlInAlert
        self.action = action
        self.isDisabled = isDisabled
        configure(with: isOn.wrappedValue)
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
        guard !controlInAlert
        else {
            action?(isOn)
            return
        }
        isOn.toggle()
        action?(isOn)
    }
}

// MARK: - Accessibility -
extension ToggleViewModel {
    func accessibilityValue() -> String {
        let value = isOn ? "general.on".localizedString : "general.off".localizedString
        if isDisabled {
            return "\(value) \("accessibility.dimmed".localizedString)"
        } else {
            return value
        }
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
