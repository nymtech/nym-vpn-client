import SwiftUI
import ImpactGenerator
import UIComponents
import Theme

struct AutologinOverlay: ViewModifier {
    var autologinState: AutologinState
    var onRetry: (() -> Void)?

    func body(content: Content) -> some View {
        @Bindable var autologin = autologinState
        content
            .overlay {
                if autologinState.isPinCodeDisplayed, !autologinState.pinCode.isEmpty {
                    PinCodeView(
                        isDisplayed: $autologin.isPinCodeDisplayed,
                        pinCode: $autologin.pinCode,
                        url: $autologin.url
                    )
                }
            }
            .overlay {
                if autologinState.isLoading {
                    ActionDialogView(
                        viewModel: ActionDialogViewModel(
                            isDisplayed: $autologin.isLoading,
                            configuration: loadingConfiguration,
                            impactGenerator: .shared,
                            isLoading: .constant(true)
                        )
                    )
                }
            }
            .overlay {
                if autologinState.isError {
                    ActionDialogView(
                        viewModel: ActionDialogViewModel(
                            isDisplayed: $autologin.isError,
                            configuration: errorConfiguration,
                            impactGenerator: .shared
                        )
                    )
                }
            }
    }

    private var loadingConfiguration: ActionDialogConfiguration {
        ActionDialogConfiguration(
            titleLocalizedString: "settings.account.autologin.fetchingPinCode".localizedString,
            noLocalizedString: "cancel".localizedString,
            noAction: {
                autologinState.cancel()
            },
            loadingText: "settings.account.autologin.loading".localizedString,
            shouldCloseAfterYesAction: false
        )
    }

    private var errorConfiguration: ActionDialogConfiguration {
        if let onRetry {
            ActionDialogConfiguration(
                systemIconImageName: "exclamationmark.triangle",
                systemIconImageColor: Color.Nym.error,
                titleLocalizedString: "generalNymError.somethingWentWrong".localizedString,
                subtitleLocalizedString: autologinState.errorMessage,
                yesLocalizedString: "settings.account.autologin.retry".localizedString,
                noLocalizedString: "cancel".localizedString,
                yesAction: {
                    autologinState.isError = false
                    onRetry()
                },
                shouldCloseAfterYesAction: false,
                verticalButtonsLayout: true
            )
        } else {
            ActionDialogConfiguration(
                systemIconImageName: "exclamationmark.triangle",
                systemIconImageColor: Color.Nym.error,
                titleLocalizedString: "generalNymError.somethingWentWrong".localizedString,
                subtitleLocalizedString: autologinState.errorMessage,
                noLocalizedString: "cancel".localizedString
            )
        }
    }
}

extension View {
    public func autologinOverlay(state: AutologinState, onRetry: (() -> Void)? = nil) -> some View {
        modifier(AutologinOverlay(autologinState: state, onRetry: onRetry))
    }
}
