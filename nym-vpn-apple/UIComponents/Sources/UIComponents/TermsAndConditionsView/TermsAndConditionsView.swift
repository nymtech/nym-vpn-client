import SwiftUI
import Constants
import Theme

public struct TermsAndConditionsView: View {
    private var privacyPolicyAttributtedString: AttributedString? {
        let terms = "welcome.termsOfUse".localizedString
        let pp1 = "welcome.privacyPolicy1".localizedString
        let pp2 = "welcome.privacyPolicy2".localizedString
        let pp = "welcome.privacyPolicy".localizedString
        let termsURL = Constants.termsOfUseURL.rawValue
        let privacyURL = Constants.privacyPolicyURL.rawValue

        let options = AttributedString.MarkdownParsingOptions(interpretedSyntax: .inlineOnlyPreservingWhitespace)

        let privacyMarkdown =
        "\(pp1) [\(terms)](\(termsURL)) \(pp2) [\(pp)](\(privacyURL))"
        return try? AttributedString(markdown: privacyMarkdown, options: options)
    }

    public var body: some View {
        Text(privacyPolicyAttributtedString ?? "")
            .tint(NymColor.primary)
            .foregroundStyle(NymColor.gray1)
            .textStyle(.Body.Small.regular)
            .multilineTextAlignment(.center)
            .padding(.horizontal, 16)
    }

    public init() {}
}
